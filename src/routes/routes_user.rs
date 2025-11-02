use crate::{
    auth::verify_jwt,
    error::{AppError, AppResult},
    routes::AppState,
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
    #[derive(Serialize, sqlx::FromRow)]
    struct UserProfileRow {
        id: Uuid,
        username: String,
        email: Option<String>,
        last_lottery_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }
    let user: UserProfileRow = sqlx::query_as(r#"SELECT id, username, email, last_lottery_at, created_at, updated_at FROM users WHERE id=$1"#)
        .bind(uid)
        .fetch_one(&state.pool)
        .await?;
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
    #[derive(Serialize, sqlx::FromRow)]
    struct Rec {
        id: Uuid,
        prize_id: Option<Uuid>,
        prize_name: Option<String>,
        created_at: DateTime<Utc>,
    }
    let records: Vec<Rec> = sqlx::query_as(r#"SELECT id, prize_id, prize_name, created_at FROM lottery_records WHERE user_id=$1 ORDER BY created_at DESC LIMIT 100"#)
        .bind(uid)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({"records": records})))
}
