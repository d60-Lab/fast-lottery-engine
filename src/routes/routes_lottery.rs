use crate::{
    auth::verify_jwt,
    error::{AppError, AppResult},
    routes::AppState,
    services::{lottery_service, prize_service},
};
use axum::{extract::State, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Serialize;
use sqlx::types::Uuid;

#[derive(serde::Serialize)]
pub struct DrawResult {
    pub won: bool,
    pub prize_id: Option<Uuid>,
    pub prize_name: Option<String>,
}

pub async fn list_prizes(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    #[derive(Serialize, sqlx::FromRow)]
    struct PrizeRow { id: Uuid, activity_id: Uuid, name: String, description: Option<String>, total_count: i64, remaining_count: i64, probability: i32, is_enabled: bool, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
    let prizes: Vec<PrizeRow> = sqlx::query_as(
        r#"SELECT id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at
           FROM prizes WHERE is_enabled=true AND remaining_count>0 ORDER BY updated_at DESC"#
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(serde_json::json!({"prizes": prizes})))
}

#[axum::debug_handler]
pub async fn draw(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<DrawResult>> {
    let claims = verify_jwt(&state.cfg, bearer.token())?;
    let uid = Uuid::parse_str(&claims.uid).map_err(|_| AppError::Unauthorized)?;

    let res = lottery_service::draw(&state.pool, uid).await?;
    Ok(Json(DrawResult { won: res.won, prize_id: res.prize_id, prize_name: res.prize_name }))
}

pub async fn global_history(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rows = lottery_service::global_history(&state.pool).await?;
    Ok(Json(serde_json::json!({"records": rows})))
}
