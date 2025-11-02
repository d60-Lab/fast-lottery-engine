use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::types::Uuid;
use sqlx::PgPool;

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
pub struct UserProfileRow {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub last_lottery_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_user(pool: &PgPool, id: Uuid, username: &str, password_hash: &str, email: &Option<String>) -> sqlx::Result<()> {
    sqlx::query(
        r#"INSERT INTO users (id, username, password_hash, email, created_at, updated_at)
           VALUES ($1,$2,$3,$4, now(), now())"#
    )
    .bind(id)
    .bind(username)
    .bind(password_hash)
    .bind(email)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(())
}

pub async fn find_user_credentials(pool: &PgPool, username: &str) -> sqlx::Result<Option<(Uuid, String)>> {
    sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, password_hash FROM users WHERE username=$1"
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn is_username_taken(pool: &PgPool, username: &str) -> sqlx::Result<bool> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE username=$1")
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}

pub async fn get_profile(pool: &PgPool, uid: Uuid) -> sqlx::Result<UserProfileRow> {
    sqlx::query_as::<_, UserProfileRow>(
        r#"SELECT id, username, email, last_lottery_at, created_at, updated_at FROM users WHERE id=$1"#
    )
    .bind(uid)
    .fetch_one(pool)
    .await
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
pub struct UserHistoryRow { pub id: Uuid, pub prize_id: Option<Uuid>, pub prize_name: Option<String>, pub created_at: DateTime<Utc> }

pub async fn get_history(pool: &PgPool, uid: Uuid) -> sqlx::Result<Vec<UserHistoryRow>> {
    sqlx::query_as::<_, UserHistoryRow>(
        r#"SELECT id, prize_id, prize_name, created_at FROM lottery_records WHERE user_id=$1 ORDER BY created_at DESC LIMIT 100"#
    )
    .bind(uid)
    .fetch_all(pool)
    .await
}
