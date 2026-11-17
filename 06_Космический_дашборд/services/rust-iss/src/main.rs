mod domain;
mod config;
use config::env_u64;
mod clients;
mod error;
mod repo;
mod handlers;
mod services;
use error::ApiError;
use handlers::*;
use services::*;
use repo::init_db;
use std::{collections::HashMap, time::Duration};

use axum::{
    extract::{Path, Query, State},

    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Serialize)]
struct Health { status: &'static str, now: DateTime<Utc> }

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    client: reqwest::Client,
    nasa_url: String,          // OSDR
    nasa_key: String,          // ключ NASA
    fallback_url: String,      // ISS where-the-iss
    every_osdr: u64,
    every_iss: u64,
    every_apod: u64,
    every_neo: u64,
    every_donki: u64,
    every_spacex: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");

    let nasa_url = std::env::var("NASA_API_URL").ok().filter(|v| !v.is_empty()).ok_or(std::env::VarError::NotPresent)
        .unwrap_or_else(|_| "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json".to_string());
    let nasa_key = std::env::var("NASA_API_KEY").unwrap_or_default();

    let fallback_url = std::env::var("WHERE_ISS_URL")
        .unwrap_or_else(|_| "https://api.wheretheiss.at/v1/satellites/25544".to_string());

    let every_osdr   = env_u64("FETCH_EVERY_SECONDS", 600);
    let every_iss    = env_u64("ISS_EVERY_SECONDS",   120);
    let every_apod   = env_u64("APOD_EVERY_SECONDS",  43200); // 12ч
    let every_neo    = env_u64("NEO_EVERY_SECONDS",   7200);  // 2ч
    let every_donki  = env_u64("DONKI_EVERY_SECONDS", 3600);  // 1ч
    let every_spacex = env_u64("SPACEX_EVERY_SECONDS",3600);

    let pool = PgPoolOptions::new().max_connections(20).connect(&db_url).await?;
    init_db(&pool).await?;

    let state = AppState {
        pool: pool.clone(),
        client: clients::new_client()?,
        nasa_url: nasa_url.clone(),
        nasa_key,
        fallback_url: fallback_url.clone(),
        every_osdr, every_iss, every_apod, every_neo, every_donki, every_spacex,
    };

    // фон OSDR
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = fetch_and_store_osdr(&st).await { error!("osdr err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.every_osdr)).await;
            }
        });
    }
    // фон ISS
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = fetch_and_store_iss(&st).await { error!("iss err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.every_iss)).await;
            }
        });
    }
    // фон APOD
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = fetch_apod(&st).await { error!("apod err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.every_apod)).await;
            }
        });
    }
    // фон NeoWs
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = fetch_neo_feed(&st).await { error!("neo err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.every_neo)).await;
            }
        });
    }
    // фон DONKI
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = fetch_donki(&st).await { error!("donki err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.every_donki)).await;
            }
        });
    }
    // фон SpaceX
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = fetch_spacex_next(&st).await { error!("spacex err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.every_spacex)).await;
            }
        });
    }

    let app = Router::new()
        // общее
        .route("/health", get(|| async { Json(Health { status: "ok", now: Utc::now() }) }))
        .with_state(state.clone())
        // ISS
        .route("/last", get(last_iss))
        .route("/fetch", get(trigger_iss))
        .route("/iss/trend", get(iss_trend))
        // OSDR
        .route("/osdr/sync", get(osdr_sync))
        .route("/osdr/list", get(osdr_list))
        // Space cache
        .route("/space/:src/latest", get(space_latest))
        .route("/space/refresh", get(space_refresh))
        .route("/space/summary", get(space_summary))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000)).await?;
    info!("rust_iss listening on 0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

