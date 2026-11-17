use super::*;

/* ---------- Фетчеры и запись ---------- */

// APOD
async fn fetch_apod_inner(st: &AppState) -> anyhow::Result<()> {
    let url = "https://api.nasa.gov/planetary/apod";
    let client = &st.client;
    let mut req = client.get(url).query(&[("thumbs","true")]);
    if !st.nasa_key.is_empty() { req = req.query(&[("api_key",&st.nasa_key)]); }
    let json: Value = clients::get_json(req).await?;
    repo::write_cache(&st.pool, "apod", json).await
}

// NeoWs
async fn fetch_neo_feed_inner(st: &AppState) -> anyhow::Result<()> {
    let today = Utc::now().date_naive();
    let start = today - chrono::Days::new(2);
    let url = "https://api.nasa.gov/neo/rest/v1/feed";
    let client = &st.client;
    let mut req = client.get(url).query(&[
        ("start_date", start.to_string()),
        ("end_date", today.to_string()),
    ]);
    if !st.nasa_key.is_empty() { req = req.query(&[("api_key",&st.nasa_key)]); }
    let json: Value = clients::get_json(req).await?;
    repo::write_cache(&st.pool, "neo", json).await
}

// DONKI объединённая
pub(crate) async fn fetch_donki(st: &AppState) -> anyhow::Result<()> {
    fetch_donki_flr(st).await?;
    fetch_donki_cme(st).await?;
    Ok(())
}
async fn fetch_donki_flr_inner(st: &AppState) -> anyhow::Result<()> {
    let (from,to) = last_days(5);
    let url = "https://api.nasa.gov/DONKI/FLR";
    let client = &st.client;
    let mut req = client.get(url).query(&[("startDate",from),("endDate",to)]);
    if !st.nasa_key.is_empty() { req = req.query(&[("api_key",&st.nasa_key)]); }
    let json: Value = clients::get_json(req).await?;
    repo::write_cache(&st.pool, "flr", json).await
}
async fn fetch_donki_cme_inner(st: &AppState) -> anyhow::Result<()> {
    let (from,to) = last_days(5);
    let url = "https://api.nasa.gov/DONKI/CME";
    let client = &st.client;
    let mut req = client.get(url).query(&[("startDate",from),("endDate",to)]);
    if !st.nasa_key.is_empty() { req = req.query(&[("api_key",&st.nasa_key)]); }
    let json: Value = clients::get_json(req).await?;
    repo::write_cache(&st.pool, "cme", json).await
}

// SpaceX
async fn fetch_spacex_next_inner(st: &AppState) -> anyhow::Result<()> {
    let url = "https://api.spacexdata.com/v4/launches/next";
    let client = &st.client;
    let json: Value = clients::get_json(client.get(url)).await?;
    repo::write_cache(&st.pool, "spacex", json).await
}

pub(crate) fn last_days(n: i64) -> (String,String) {
    let to = Utc::now().date_naive();
    let from = to - chrono::Days::new(n as u64);
    (from.to_string(), to.to_string())
}

/* ---------- Вспомогательное ---------- */
pub(crate) fn s_pick(v: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() { if !s.is_empty() { return Some(s.to_string()); } }
            else if x.is_number() { return Some(x.to_string()); }
        }
    }
    None
}
pub(crate) fn t_pick(v: &Value, keys: &[&str]) -> Option<DateTime<Utc>> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() {
                if let Ok(dt) = s.parse::<DateTime<Utc>>() { return Some(dt); }
                if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                    return Some(Utc.from_utc_datetime(&ndt));
                }
            } else if let Some(n) = x.as_i64() {
                return Some(Utc.timestamp_opt(n, 0).single().unwrap_or_else(Utc::now));
            }
        }
    }
    None
}
async fn fetch_and_store_iss_inner(st: &AppState) -> anyhow::Result<()> {
    let pool = &st.pool;
    let url = &st.fallback_url;
    let client = &st.client;
    let json = clients::get_json(client.get(url)).await?;
    repo::insert_iss(pool, url, json).await?;
    Ok(())
}
async fn fetch_and_store_osdr_inner(st: &AppState) -> anyhow::Result<usize> {
    let client = &st.client;
    let json = clients::get_json(client.get(&st.nasa_url)).await?;
    let items = if let Some(a) = json.as_array() { a.clone() }
        else if let Some(v) = json.get("items").and_then(|x| x.as_array()) { v.clone() }
        else if let Some(v) = json.get("results").and_then(|x| x.as_array()) { v.clone() }
        else { vec![json.clone()] };

    let written = repo::store_osdr(&st.pool, items).await?;
    Ok(written)
}

pub(crate) async fn fetch_and_store_iss(st: &AppState) -> anyhow::Result<()> {
    let mut lock = st.pool.begin().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(7000_i64).fetch_one(&mut *lock).await?;
    if !acquired { return Ok(()); }
    let result = fetch_and_store_iss_inner(st).await;
    lock.rollback().await?;
    result
}

pub(crate) async fn fetch_and_store_osdr(st: &AppState) -> anyhow::Result<usize> {
    let mut lock = st.pool.begin().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(7001_i64).fetch_one(&mut *lock).await?;
    if !acquired { return Ok(0); }
    let result = fetch_and_store_osdr_inner(st).await;
    lock.rollback().await?;
    result
}

pub(crate) async fn fetch_apod(st: &AppState) -> anyhow::Result<()> {
    let mut lock = st.pool.begin().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(7002_i64).fetch_one(&mut *lock).await?;
    if !acquired { return Ok(()); }
    let result = fetch_apod_inner(st).await;
    lock.rollback().await?;
    result
}

pub(crate) async fn fetch_neo_feed(st: &AppState) -> anyhow::Result<()> {
    let mut lock = st.pool.begin().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(7003_i64).fetch_one(&mut *lock).await?;
    if !acquired { return Ok(()); }
    let result = fetch_neo_feed_inner(st).await;
    lock.rollback().await?;
    result
}

pub(crate) async fn fetch_donki_flr(st: &AppState) -> anyhow::Result<()> {
    let mut lock = st.pool.begin().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(7004_i64).fetch_one(&mut *lock).await?;
    if !acquired { return Ok(()); }
    let result = fetch_donki_flr_inner(st).await;
    lock.rollback().await?;
    result
}

pub(crate) async fn fetch_donki_cme(st: &AppState) -> anyhow::Result<()> {
    let mut lock = st.pool.begin().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(7005_i64).fetch_one(&mut *lock).await?;
    if !acquired { return Ok(()); }
    let result = fetch_donki_cme_inner(st).await;
    lock.rollback().await?;
    result
}

pub(crate) async fn fetch_spacex_next(st: &AppState) -> anyhow::Result<()> {
    let mut lock = st.pool.begin().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(7006_i64).fetch_one(&mut *lock).await?;
    if !acquired { return Ok(()); }
    let result = fetch_spacex_next_inner(st).await;
    lock.rollback().await?;
    result
}
