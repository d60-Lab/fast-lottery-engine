use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{types::Uuid, PgPool, Postgres, Transaction, Row};
use crate::models::Prize;

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
pub struct EnabledPrize { pub id: Uuid, pub name: String, pub remaining_count: i64, pub probability: i32 }

pub async fn list_enabled_prizes(pool: &PgPool) -> sqlx::Result<Vec<EnabledPrize>> {
    sqlx::query_as::<_, EnabledPrize>(
        r#"SELECT id, name, remaining_count, probability FROM prizes WHERE is_enabled=true AND remaining_count>0"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_prizes(pool: &PgPool) -> sqlx::Result<Vec<Prize>> {
    sqlx::query_as::<_, Prize>(
        r#"SELECT id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at FROM prizes ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
}

pub async fn create_prize(
    pool: &PgPool,
    id: Uuid,
    activity_id: Uuid,
    name: String,
    description: Option<String>,
    total_count: i64,
    probability: i32,
    is_enabled: bool,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"INSERT INTO prizes (id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at)
           VALUES ($1,$2,$3,$4,$5,$5,$6,$7, now(), now())"#
    )
    .bind(id)
    .bind(activity_id)
    .bind(name)
    .bind(description)
    .bind(total_count)
    .bind(probability)
    .bind(is_enabled)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn decrement_stock(tx: &mut Transaction<'_, Postgres>, prize_id: Uuid) -> sqlx::Result<Option<i64>> {
    let row = sqlx::query(
        r#"UPDATE prizes SET remaining_count = remaining_count - 1, updated_at=now()
            WHERE id=$1 AND remaining_count>0 RETURNING remaining_count"#
    )
    .bind(prize_id)
    .fetch_optional(&mut *(*tx))
    .await?;
    Ok(row.map(|r| r.get::<i64, _>("remaining_count")))
}
