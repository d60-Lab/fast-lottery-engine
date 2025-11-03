use crate::{
    config::Config,
    error::{AppError, AppResult},
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub uid: String,
    pub exp: usize,
    pub is_admin: bool,
}

pub fn now_ts() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs() as usize
}

pub fn sign_jwt(cfg: &Config, uid: &str, is_admin: bool) -> AppResult<String> {
    let claims = Claims {
        sub: if is_admin {
            "admin".into()
        } else {
            "user".into()
        },
        uid: uid.into(),
        exp: now_ts() + 60 * 60 * 24 * 7,
        is_admin,
    };
    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal("jwt encode failed"))?;
    Ok(token)
}

pub fn verify_jwt(cfg: &Config, token: &str) -> AppResult<Claims> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

pub fn hash_password(pw: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(pw.as_bytes(), &salt)
        .map_err(|_| AppError::Internal("password hash failed"))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(pw: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash).map_err(|_| AppError::Internal("bad password hash"))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(pw.as_bytes(), &parsed).is_ok())
}

// custom extractor removed; use explicit header + state in handlers
