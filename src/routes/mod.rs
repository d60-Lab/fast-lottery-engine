
use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;

pub type AppState = Arc<StateData>;

#[derive(Clone)]
pub struct StateData {
    pub pool: PgPool,
    pub cfg: Config,
}

pub fn auth_routes(pool: &PgPool, cfg: &Config) -> Router {
    let state = Arc::new(StateData {
        pool: pool.clone(),
        cfg: cfg.clone(),
    });
    Router::new()
        .route("/api/auth/register", post(self::routes_auth::register))
        .route("/api/auth/login", post(self::routes_auth::login))
        .with_state(state)
}

pub fn user_routes(pool: &PgPool) -> Router {
    let state = Arc::new(StateData {
        pool: pool.clone(),
        cfg: Config::from_env().expect("cfg"),
    });
    Router::new()
        .route("/api/user/profile", get(self::routes_user::profile))
        .route("/api/user/lottery-history", get(self::routes_user::history))
        .with_state(state)
}

pub fn lottery_routes(pool: &PgPool) -> Router {
    let state = Arc::new(StateData {
        pool: pool.clone(),
        cfg: Config::from_env().expect("cfg"),
    });
    Router::new()
        .route("/api/lottery/draw", post(self::routes_lottery::draw))
        .route(
            "/api/lottery/prizes",
            get(self::routes_lottery::list_prizes),
        )
        .route(
            "/api/lottery/global-history",
            get(self::routes_lottery::global_history),
        )
        .with_state(state)
}

pub fn admin_routes(pool: &PgPool, cfg: &Config) -> Router {
    let state = Arc::new(StateData {
        pool: pool.clone(),
        cfg: cfg.clone(),
    });
    Router::new()
        .route("/admin/api/login", post(self::routes_admin::admin_login))
        .route(
            "/admin/api/activities",
            get(self::routes_admin::list_activities).post(self::routes_admin::create_activity),
        )
        .route(
            "/admin/api/prizes",
            get(self::routes_admin::list_prizes).post(self::routes_admin::create_prize),
        )
        .with_state(state)
}

pub mod routes_admin;
pub mod routes_auth;
pub mod routes_lottery;
pub mod routes_user;
