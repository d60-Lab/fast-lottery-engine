use crate::{
    auth::verify_jwt,
    error::{AppError, AppResult},
    routes::AppState,
    services::user_service,
};
use axum::{extract::State, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::types::Uuid;

pub async fn profile(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = verify_jwt(&state.cfg, bearer.token())?;
    let uid = Uuid::parse_str(&claims.uid).map_err(|_| AppError::Unauthorized)?;
    let user = user_service::get_profile(&state.pool, uid).await?;
    Ok(Json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "last_lottery_at": user.last_lottery_at,
        "created_at": user.created_at,
        "updated_at": user.updated_at,
    })))
}

pub async fn history(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = verify_jwt(&state.cfg, bearer.token())?;
    let uid = Uuid::parse_str(&claims.uid).map_err(|_| AppError::Unauthorized)?;
    let records = user_service::get_history(&state.pool, uid).await?;
    Ok(Json(serde_json::json!({"records": records})))
}
