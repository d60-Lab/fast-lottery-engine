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
mod error;
mod models;
mod routes;
mod services;

use crate::{
    config::Config,
    db::connect_pool,
    routes::{admin_routes, auth_routes, lottery_routes, user_routes},
};

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
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
