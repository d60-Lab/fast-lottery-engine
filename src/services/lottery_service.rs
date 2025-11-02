use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Serialize;
use sqlx::{types::Uuid, PgPool};

use crate::error::AppError;
use crate::services::prize_service::{decrement_stock, EnabledPrize};

#[derive(Serialize, Debug)]
pub struct DrawResult { pub won: bool, pub prize_id: Option<Uuid>, pub prize_name: Option<String> }

pub async fn list_enabled_prizes(pool: &PgPool) -> sqlx::Result<Vec<EnabledPrize>> {
    super::prize_service::list_enabled_prizes(pool).await
}

pub async fn global_history(pool: &PgPool) -> sqlx::Result<Vec<GlobalRecordRow>> {
    sqlx::query_as::<_, GlobalRecordRow>(
        r#"SELECT id, user_id, prize_id, prize_name, created_at FROM lottery_records ORDER BY created_at DESC LIMIT 200"#
    )
    .fetch_all(pool)
    .await
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
pub struct GlobalRecordRow { pub id: Uuid, pub user_id: Uuid, pub prize_id: Option<Uuid>, pub prize_name: Option<String>, pub created_at: DateTime<Utc> }

pub async fn draw(pool: &PgPool, uid: Uuid) -> Result<DrawResult, AppError> {
    let mut tx = pool.begin().await?;

    let last: Option<DateTime<Utc>> = sqlx::query_scalar("SELECT last_lottery_at FROM users WHERE id=$1 FOR UPDATE")
        .bind(uid)
        .fetch_one(&mut *tx)
        .await?;
    if let Some(last) = last {
        let seconds = (Utc::now() - last).num_seconds();
        if seconds < 60 { return Err(AppError::BadRequest("抽奖频率过高，请稍后再试")); }
    }

    let prizes = sqlx::query_as::<_, EnabledPrize>(
        r#"SELECT id, name, remaining_count, probability FROM prizes
           WHERE is_enabled=true AND remaining_count>0"#
    )
    .fetch_all(&mut *tx)
    .await?;

    let total_weight: i32 = prizes.iter().map(|p| p.probability.max(0)).sum();
    let no_win_weight = (100 - total_weight).max(0);
    let roll = { let mut rng = rand::thread_rng(); rng.gen_range(1..=std::cmp::max(1, total_weight + no_win_weight)) };

    let mut acc = 0;
    let mut selected: Option<(Uuid, String)> = None;
    for p in &prizes {
        acc += p.probability.max(0);
        if roll <= acc { selected = Some((p.id, p.name.clone())); break; }
    }

    let (won, prize_id, prize_name) = if let Some((pid, pname)) = selected {
        let updated = decrement_stock(&mut tx, pid).await?;
        if updated.is_some() { (true, Some(pid), Some(pname)) } else { (false, None, None) }
    } else { (false, None, None) };

    sqlx::query(
        r#"INSERT INTO lottery_records (id, user_id, prize_id, prize_name, created_at)
            VALUES ($1,$2,$3,$4, now())"#
    )
    .bind(Uuid::new_v4())
    .bind(uid)
    .bind(prize_id)
    .bind(prize_name.as_deref())
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE users SET last_lottery_at=now(), updated_at=now() WHERE id=$1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(DrawResult { won, prize_id, prize_name })
}
