
use crate::{
    auth::{hash_password, sign_jwt, verify_password},
    error::{AppError, AppResult},
    models::{JwtResponse, LoginDto, RegisterDto},
    routes::AppState,
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
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE username=$1")
        .bind(&payload.username)
        .fetch_optional(&state.pool)
        .await?;
    if exists.is_some() {
        return Err(AppError::BadRequest("用户名已存在"));
    }

    let hash = hash_password(&payload.password)?;
    let uid = Uuid::new_v4();
    sqlx::query(
        r#"
         INSERT INTO users (id, username, password_hash, email, created_at, updated_at)
         VALUES ($1,$2,$3,$4, now(), now())
     "#,
    )
    .bind(uid)
    .bind(&payload.username)
    .bind(hash)
    .bind(&payload.email)
    .execute(&state.pool)
    .await?;

    let token = sign_jwt(&state.cfg, &uid.to_string(), false)?;
    Ok(Json(JwtResponse { token }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginDto>,
) -> AppResult<Json<JwtResponse>> {
    let row = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, password_hash FROM users WHERE username=$1",
    )
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await?;
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
