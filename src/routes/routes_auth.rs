
use crate::{
    auth::{hash_password, sign_jwt, verify_password},
    error::{AppError, AppResult},
    models::{JwtResponse, LoginDto, RegisterDto},
    routes::AppState,
    services::user_service,
};
use axum::{extract::State, Json};
use sqlx::types::Uuid;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterDto>,
) -> AppResult<Json<JwtResponse>> {
    if payload.username.trim().is_empty() || payload.password.len() < 6 {
        return Err(AppError::BadRequest("用户名或密码不合法"));
    }
    if user_service::is_username_taken(&state.pool, &payload.username).await? {
        return Err(AppError::BadRequest("用户名已存在"));
    }

    let hash = hash_password(&payload.password)?;
    let uid = Uuid::new_v4();
    user_service::create_user(&state.pool, uid, &payload.username, &hash, &payload.email).await?;

    let token = sign_jwt(&state.cfg, &uid.to_string(), false)?;
    Ok(Json(JwtResponse { token }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> AppResult<Json<JwtResponse>> {
    let row = user_service::find_user_credentials(&state.pool, &payload.username).await?;
    let (uid, pw_hash) = match row {
        Some(v) => v,
        None => return Err(AppError::Unauthorized),
    };
    if !verify_password(&payload.password, &pw_hash)? {
        return Err(AppError::Unauthorized);
    }
    let token = sign_jwt(&state.cfg, &uid.to_string(), false)?;
    Ok(Json(JwtResponse { token }))
}
