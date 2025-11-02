
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub last_lottery_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Activity {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: ActivityStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "activity_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ActivityStatus {
    Planned,
    Ongoing,
    Paused,
    Ended,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Prize {
    pub id: Uuid,
    pub activity_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub total_count: i64,
    pub remaining_count: i64,
    pub probability: i32, // 作为权重
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct LotteryRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub prize_id: Option<Uuid>,
    pub prize_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDto {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct JwtResponse {
    pub token: String,
}
