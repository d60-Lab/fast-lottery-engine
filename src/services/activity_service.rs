use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Activity, ActivityStatus};
use chrono::{DateTime, Utc};

pub async fn list_activities(pool: &PgPool) -> sqlx::Result<Vec<Activity>> {
    sqlx::query_as::<_, Activity>(
        r#"SELECT id, name, description, start_time, end_time, status, created_at, updated_at FROM activities ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
}

pub async fn create_activity(
    pool: &PgPool,
    id: Uuid,
    name: String,
    description: Option<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    status: ActivityStatus,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"INSERT INTO activities (id, name, description, start_time, end_time, status, created_at, updated_at)
           VALUES ($1,$2,$3,$4,$5,$6, now(), now())"#
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(start_time)
    .bind(end_time)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}
