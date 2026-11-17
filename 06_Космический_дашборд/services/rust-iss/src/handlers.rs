use super::*;

/* ---------- ISS ---------- */
pub(crate) async fn last_iss(State(st): State<AppState>)
-> Result<Json<Value>, ApiError> {
    let row_opt = repo::last_iss_row(&st.pool).await
     .map_err(|e| ApiError::internal(e))?;

    if let Some(row) = row_opt {
        let id: i64 = row.get("id");
        let fetched_at: DateTime<Utc> = row.get::<DateTime<Utc>, _>("fetched_at");
        let source_url: String = row.get("source_url");
        let payload: Value = row.try_get("payload").unwrap_or(serde_json::json!({}));
        return Ok(Json(serde_json::json!({
            "id": id, "fetched_at": fetched_at, "source_url": source_url, "payload": payload
        })));
    }
    Ok(Json(serde_json::json!({"message":"no data"})))
}

pub(crate) async fn trigger_iss(State(st): State<AppState>)
-> Result<Json<Value>, ApiError> {
    fetch_and_store_iss(&st).await
        .map_err(|e| ApiError::internal(e))?;
    last_iss(State(st)).await
}

#[derive(Serialize)]
pub(crate) struct Trend {
    movement: bool,
    delta_km: f64,
    dt_sec: f64,
    velocity_kmh: Option<f64>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    from_lat: Option<f64>,
    from_lon: Option<f64>,
    to_lat: Option<f64>,
    to_lon: Option<f64>,
}

pub(crate) async fn iss_trend(State(st): State<AppState>)
-> Result<Json<Trend>, ApiError> {
    let rows = repo::last_two_iss_rows(&st.pool).await
        .map_err(|e| ApiError::internal(e))?;

    if rows.len() < 2 {
        return Ok(Json(Trend {
            movement: false, delta_km: 0.0, dt_sec: 0.0, velocity_kmh: None,
            from_time: None, to_time: None,
            from_lat: None, from_lon: None, to_lat: None, to_lon: None
        }));
    }

    let t2: DateTime<Utc> = rows[0].get("fetched_at");
    let t1: DateTime<Utc> = rows[1].get("fetched_at");
    let p2: Value = rows[0].get("payload");
    let p1: Value = rows[1].get("payload");

    let lat1 = domain::num(&p1["latitude"]);
    let lon1 = domain::num(&p1["longitude"]);
    let lat2 = domain::num(&p2["latitude"]);
    let lon2 = domain::num(&p2["longitude"]);
    let v2 = domain::num(&p2["velocity"]);

    let mut delta_km = 0.0;
    let mut movement = false;
    if let (Some(a1), Some(o1), Some(a2), Some(o2)) = (lat1, lon1, lat2, lon2) {
        delta_km = domain::haversine_km(a1, o1, a2, o2);
        movement = delta_km > 0.1;
    }
    let dt_sec = (t2 - t1).num_milliseconds() as f64 / 1000.0;

    Ok(Json(Trend {
        movement,
        delta_km,
        dt_sec,
        velocity_kmh: v2,
        from_time: Some(t1),
        to_time: Some(t2),
        from_lat: lat1, from_lon: lon1, to_lat: lat2, to_lon: lon2,
    }))
}

/* ---------- OSDR ---------- */
pub(crate) async fn osdr_sync(State(st): State<AppState>)
-> Result<Json<Value>, ApiError> {
    let written = fetch_and_store_osdr(&st).await
        .map_err(|e| ApiError::internal(e))?;
    Ok(Json(serde_json::json!({ "written": written })))
}

pub(crate) async fn osdr_list(Query(query): Query<HashMap<String,String>>, State(st): State<AppState>)
-> Result<Json<Value>, ApiError> {
    let limit = query.get("limit").cloned().or_else(|| std::env::var("OSDR_LIST_LIMIT").ok())
        .and_then(|s| s.parse::<i64>().ok()).unwrap_or(20).clamp(1, 100);

    let rows = repo::list_osdr_rows(&st.pool, limit).await
     .map_err(|e| ApiError::internal(e))?;

    let out: Vec<Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.get::<i64,_>("id"),
            "dataset_id": r.get::<Option<String>,_>("dataset_id"),
            "title": r.get::<Option<String>,_>("title"),
            "status": r.get::<Option<String>,_>("status"),
            "updated_at": r.get::<Option<DateTime<Utc>>,_>("updated_at"),
            "inserted_at": r.get::<DateTime<Utc>, _>("inserted_at"),
            "raw": r.get::<Value,_>("raw"),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "items": out })))
}

/* ---------- Универсальная витрина space_cache ---------- */

pub(crate) async fn space_latest(Path(src): Path<String>, State(st): State<AppState>)
-> Result<Json<Value>, ApiError> {
    let row = repo::latest_space_row(&st.pool, &src).await
     .map_err(|e| ApiError::internal(e))?;

    if let Some(r) = row {
        let fetched_at: DateTime<Utc> = r.get("fetched_at");
        let payload: Value = r.get("payload");
        return Ok(Json(serde_json::json!({ "source": src, "fetched_at": fetched_at, "payload": payload })));
    }
    Ok(Json(serde_json::json!({ "source": src, "message":"no data" })))
}

pub(crate) async fn space_refresh(Query(q): Query<HashMap<String,String>>, State(st): State<AppState>)
-> Result<Json<Value>, ApiError> {
    let list = q.get("src").cloned().unwrap_or_else(|| "apod,neo,flr,cme,spacex".to_string());
    let mut done = Vec::new();
    for s in list.split(',').map(|x| x.trim().to_lowercase()) {
        match s.as_str() {
            "apod"   => { fetch_apod(&st).await.map_err(ApiError::internal)?;       done.push("apod"); }
            "neo"    => { fetch_neo_feed(&st).await.map_err(ApiError::internal)?;   done.push("neo"); }
            "flr"    => { fetch_donki_flr(&st).await.map_err(ApiError::internal)?;  done.push("flr"); }
            "cme"    => { fetch_donki_cme(&st).await.map_err(ApiError::internal)?;  done.push("cme"); }
            "spacex" => { fetch_spacex_next(&st).await.map_err(ApiError::internal)?; done.push("spacex"); }
            _ => {}
        }
    }
    Ok(Json(serde_json::json!({ "refreshed": done })))
}

pub(crate) async fn space_summary(State(st): State<AppState>)
-> Result<Json<Value>, ApiError> {
    let rows = repo::all_latest_space_rows(&st.pool).await.map_err(ApiError::internal)?;
    let mut latest = HashMap::new();
    for row in rows {
        let source: String = row.get("source");
        latest.insert(source, serde_json::json!({"at":row.get::<DateTime<Utc>,_>("fetched_at"),
            "payload":row.get::<Value,_>("payload")}));
    }
    let empty = serde_json::json!({});
    let apod = latest.get("apod").unwrap_or(&empty);
    let neo = latest.get("neo").unwrap_or(&empty);
    let flr = latest.get("flr").unwrap_or(&empty);
    let cme = latest.get("cme").unwrap_or(&empty);
    let spacex = latest.get("spacex").unwrap_or(&empty);
    let iss_last = repo::summary_iss_row(&st.pool).await.map_err(ApiError::internal)?
        .map(|r| serde_json::json!({"at":r.get::<DateTime<Utc>,_>("fetched_at"),
            "payload":r.get::<Value,_>("payload")})).unwrap_or(serde_json::json!({}));
    let osdr_count: i64 = repo::count_osdr_row(&st.pool).await.map_err(ApiError::internal)?.get("c");

    Ok(Json(serde_json::json!({
        "apod": apod, "neo": neo, "flr": flr, "cme": cme, "spacex": spacex,
        "iss": iss_last, "osdr_count": osdr_count
    })))
}

