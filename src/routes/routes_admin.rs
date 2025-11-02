
use crate::{
    auth::{sign_jwt, verify_jwt},
    error::{AppError, AppResult},
    models::{Activity, Prize},
    routes::AppState,
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
    let rows: Vec<Activity> = sqlx::query_as(r#"SELECT id, name, description, start_time, end_time, status::text as status, created_at, updated_at FROM activities ORDER BY created_at DESC"#)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({"activities": rows})))
}

#[derive(Deserialize)]
pub struct CreateActivityDto {
    pub name: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: String,
}

pub async fn create_activity(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<CreateActivityDto>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, bearer.token())?;
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO activities (id, name, description, start_time, end_time, status, created_at, updated_at)
           VALUES ($1,$2,$3,$4,$5, CAST($6 AS activity_status), now(), now())"#
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.start_time)
    .bind(payload.end_time)
    .bind(payload.status)
    .execute(&state.pool)
    .await?;
    Ok(Json(serde_json::json!({"id": id})))
}

pub async fn list_prizes(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, bearer.token())?;
    let rows: Vec<Prize> = sqlx::query_as(r#"SELECT id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at FROM prizes ORDER BY created_at DESC"#)
        .fetch_all(&state.pool)
        .await?;
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
    sqlx::query(
        r#"INSERT INTO prizes (id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at)
           VALUES ($1,$2,$3,$4,$5,$5,$6,$7, now(), now())"#
    )
    .bind(id)
    .bind(payload.activity_id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.total_count)
    .bind(payload.probability)
    .bind(payload.is_enabled)
    .execute(&state.pool)
    .await?;
    Ok(Json(serde_json::json!({"id": id})))
}
