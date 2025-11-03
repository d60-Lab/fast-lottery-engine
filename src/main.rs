use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod db;
mod redis_scripts;
mod redis_client;
mod error;
mod models;
mod routes;
mod services;

use crate::{
    config::Config,
    db::connect_pool,
    routes::{admin_routes, auth_routes, lottery_routes, user_routes},
};
use std::sync::Arc;
use crate::services::stock_sync::spawn_redis_delta_flusher;
use crate::services::prize_cache;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = Config::from_env()?;
    let pool = connect_pool(&cfg.database_url).await?;
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_mgr = crate::redis_client::connect_manager(&redis_url).await?;
    let redis = Arc::new(redis_mgr);

    let api = Router::new()
        .merge(auth_routes(&pool, &cfg))
        .merge(user_routes(&pool, &cfg))
        .merge(lottery_routes(&pool, &cfg))
        .merge(admin_routes(&pool, &cfg))
        .route("/healthz", get(|| async { "ok" }));

    let app = Router::new()
        .nest("/", api)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = cfg.server_addr.parse()?;
    tracing::info!(%addr, "server starting");
    // spawn background flusher for Redis deltas to DB
    spawn_redis_delta_flusher(pool.clone(), redis.clone());
    // spawn prize cache refresher to avoid DB read per draw
    prize_cache::spawn_refresh(pool.clone());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
