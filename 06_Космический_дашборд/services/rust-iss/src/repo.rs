use super::*;

/* ---------- DB boot ---------- */
pub(crate) async fn init_db(pool: &PgPool) -> anyhow::Result<()> {
    // ISS
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS iss_fetch_log(
            id BIGSERIAL PRIMARY KEY,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            source_url TEXT NOT NULL,
            payload JSONB NOT NULL
        )"
    ).execute(pool).await?;

    // OSDR
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS osdr_items(
            id BIGSERIAL PRIMARY KEY,
            dataset_id TEXT,
            title TEXT,
            status TEXT,
            updated_at TIMESTAMPTZ,
            inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            raw JSONB NOT NULL
        )"
    ).execute(pool).await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id
         ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL"
    ).execute(pool).await?;

    // СѓРЅРёРІРµСЂСЃР°Р»СЊРЅС‹Р№ РєСЌС€ РєРѕСЃРјРѕРґР°РЅРЅС‹С…
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS space_cache(
            id BIGSERIAL PRIMARY KEY,
            source TEXT NOT NULL,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            payload JSONB NOT NULL
        )"
    ).execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS ix_space_cache_source ON space_cache(source,id DESC)").execute(pool).await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS ix_space_latest ON space_cache(source,id DESC)").execute(pool).await?;
    Ok(())
}


pub(crate) async fn last_iss_row(pool: &PgPool) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        "SELECT id, fetched_at, source_url, payload
         FROM iss_fetch_log
         ORDER BY id DESC LIMIT 1"
    ).fetch_optional(pool).await
}

pub(crate) async fn last_two_iss_rows(pool: &PgPool) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query("SELECT fetched_at, payload FROM iss_fetch_log ORDER BY id DESC LIMIT 2")
        .fetch_all(pool).await
}

pub(crate) async fn list_osdr_rows(pool: &PgPool, limit: i64) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
         FROM osdr_items
         ORDER BY inserted_at DESC
         LIMIT $1"
    ).bind(limit).fetch_all(pool).await
}

pub(crate) async fn latest_space_row(pool: &PgPool, src: &str) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        "SELECT fetched_at, payload FROM space_cache
         WHERE source = $1 ORDER BY id DESC LIMIT 1"
    ).bind(src).fetch_optional(pool).await
}

pub(crate) async fn cached_space_row(pool: &PgPool, src: &str) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query("SELECT fetched_at, payload FROM space_cache WHERE source=$1 ORDER BY id DESC LIMIT 1")
        .bind(src)
        .fetch_optional(pool).await
}

pub(crate) async fn summary_iss_row(pool: &PgPool) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query("SELECT fetched_at,payload FROM iss_fetch_log ORDER BY id DESC LIMIT 1")
        .fetch_optional(pool).await
}

pub(crate) async fn count_osdr_row(pool: &PgPool) -> Result<sqlx::postgres::PgRow, sqlx::Error> {
    sqlx::query("SELECT count(*) AS c FROM osdr_items")
        .fetch_one(pool).await
}

pub(crate) async fn write_cache(pool: &PgPool, source: &str, payload: Value) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO space_cache(source, payload) VALUES ($1,$2)")
        .bind(source).bind(payload).execute(pool).await?;
    Ok(())
}


pub(crate) async fn insert_iss(pool: &PgPool, url: &str, json: Value) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2)")
        .bind(url).bind(json).execute(pool).await?;
    Ok(())
}

pub(crate) async fn store_osdr(pool: &PgPool, items: Vec<Value>) -> anyhow::Result<usize> {
    let mut written = 0usize;
    for item in items {
        let id = s_pick(&item, &["dataset_id","id","uuid","studyId","accession","osdr_id"]);
        let title = s_pick(&item, &["title","name","label"]);
        let status = s_pick(&item, &["status","state","lifecycle"]);
        let updated = t_pick(&item, &["updated","updated_at","modified","lastUpdated","timestamp"]);
        if let Some(ds) = id.clone() {
            sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1,$2,$3,$4,$5)
                 ON CONFLICT (dataset_id) WHERE dataset_id IS NOT NULL DO UPDATE
                 SET title=EXCLUDED.title, status=EXCLUDED.status,
                     updated_at=EXCLUDED.updated_at, raw=EXCLUDED.raw"
            ).bind(ds).bind(title).bind(status).bind(updated).bind(item).execute(pool).await?;
        } else {
            sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1,$2,$3,$4,$5)"
            ).bind::<Option<String>>(None).bind(title).bind(status).bind(updated).bind(item).execute(pool).await?;
        }
        written += 1;
    }
    Ok(written)
}

pub(crate) async fn all_latest_space_rows(pool: &PgPool) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query("SELECT DISTINCT ON (source) source, fetched_at, payload FROM space_cache ORDER BY source, id DESC")
        .fetch_all(pool).await
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    async fn test_pool() -> PgPool {
        let url = std::env::var("TEST_DATABASE_URL").expect("Run tests through Docker Compose");
        PgPoolOptions::new().max_connections(3).connect(&url).await.unwrap()
    }

    #[tokio::test]
    async fn osdr_upsert_keeps_one_row_and_utc_timestamp() {
        let pool = test_pool().await;
        init_db(&pool).await.unwrap();
        let id = format!("test-{}", Utc::now().timestamp_micros());
        for title in ["Before", "After"] {
            store_osdr(&pool, vec![serde_json::json!({
                "dataset_id":id,"title":title,"updated_at":"2026-11-17T10:00:00Z"
            })]).await.unwrap();
        }
        let rows = sqlx::query("SELECT title, updated_at FROM osdr_items WHERE dataset_id=$1")
            .bind(&id).fetch_all(&pool).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<String,_>("title"), "After");
        assert_eq!(rows[0].get::<DateTime<Utc>,_>("updated_at").to_rfc3339(), "2026-11-17T10:00:00+00:00");
        sqlx::query("DELETE FROM osdr_items WHERE dataset_id=$1").bind(&id).execute(&pool).await.unwrap();
        pool.close().await;
    }

    #[tokio::test]
    async fn advisory_lock_prevents_overlap_and_is_released() {
        let pool = test_pool().await;
        let mut first = pool.begin().await.unwrap();
        let mut second = pool.begin().await.unwrap();
        let query = "SELECT pg_try_advisory_xact_lock(990066)";
        assert!(sqlx::query_scalar::<_,bool>(query).fetch_one(&mut *first).await.unwrap());
        assert!(!sqlx::query_scalar::<_,bool>(query).fetch_one(&mut *second).await.unwrap());
        first.rollback().await.unwrap();
        assert!(sqlx::query_scalar::<_,bool>(query).fetch_one(&mut *second).await.unwrap());
        second.rollback().await.unwrap();
        pool.close().await;
    }
}
