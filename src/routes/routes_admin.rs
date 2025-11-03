
use crate::{
    auth::{sign_jwt, verify_jwt},
    error::{AppError, AppResult},
    models::{Activity, Prize, ActivityStatus},
    routes::AppState,
    services::{activity_service, prize_service},
};
use axum::{extract::State, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::types::Uuid;

#[derive(Deserialize)]
pub struct AdminLoginDto {
    pub username: String,
    pub password: String,
}

pub async fn admin_login(
    State(state): State<AppState>,
    Json(payload): Json<AdminLoginDto>,
) -> AppResult<Json<serde_json::Value>> {
    if payload.username != state.cfg.admin_username || payload.password != state.cfg.admin_password
    {
        return Err(AppError::Unauthorized);
    }
    let token = sign_jwt(&state.cfg, "admin", true)?;
    Ok(Json(serde_json::json!({"token": token})))
}

fn ensure_admin(state: &AppState, token: &str) -> AppResult<()> {
    let claims = verify_jwt(&state.cfg, token)?;
    if !claims.is_admin {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub async fn list_activities(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, bearer.token())?;
    let rows: Vec<Activity> = activity_service::list_activities(&state.pool).await?;
    Ok(Json(serde_json::json!({"activities": rows})))
}

#[derive(Deserialize)]
pub struct CreateActivityDto {
    pub name: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: ActivityStatus,
}

pub async fn create_activity(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<CreateActivityDto>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, bearer.token())?;
    let id = Uuid::new_v4();
    activity_service::create_activity(&state.pool, id, payload.name, payload.description, payload.start_time, payload.end_time, payload.status).await?;
    Ok(Json(serde_json::json!({"id": id})))
}

pub async fn list_prizes(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, bearer.token())?;
    let rows: Vec<Prize> = prize_service::list_prizes(&state.pool).await?;
    Ok(Json(serde_json::json!({"prizes": rows})))
}

#[derive(Deserialize)]
pub struct CreatePrizeDto {
    pub activity_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub total_count: i64,
    pub probability: i32,
    pub is_enabled: bool,
}

pub async fn create_prize(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<CreatePrizeDto>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, bearer.token())?;
    let id = Uuid::new_v4();
    prize_service::create_prize(&state.pool, id, payload.activity_id, payload.name, payload.description, payload.total_count, payload.probability, payload.is_enabled).await?;
    Ok(Json(serde_json::json!({"id": id})))
}

#[derive(Deserialize)]
pub struct BenchMintReq { pub count: usize, pub prefix: Option<String> }

pub async fn bench_mint_tokens(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<BenchMintReq>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, bearer.token())?;
    let prefix = payload.prefix.unwrap_or_else(|| "bench_user_".to_string());
    let mut tokens = Vec::with_capacity(payload.count);
    let mut tx = state.pool.begin().await?;
    for i in 0..payload.count {
        let uname = format!("{}{}", prefix, i);
        // upsert user quickly with placeholder hash
        let uid: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE username=$1")
            .bind(&uname)
            .fetch_optional(&mut *tx)
            .await?;
        let uid = match uid {
            Some(id) => id,
            None => {
                let id = Uuid::new_v4();
                sqlx::query(r#"INSERT INTO users (id, username, password_hash, email, created_at, updated_at)
                    VALUES ($1,$2,$3,$4, now(), now())"#)
                    .bind(id)
                    .bind(&uname)
                    .bind("BENCH")
                    .bind(Option::<String>::None)
                    .execute(&mut *tx)
                    .await?;
                id
            }
        };
        let token = sign_jwt(&state.cfg, &uid.to_string(), false)?;
        tokens.push(token);
    }
    tx.commit().await?;
    Ok(Json(serde_json::json!({"tokens": tokens})))
}
