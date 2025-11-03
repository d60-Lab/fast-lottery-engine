use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Serialize;
use sqlx::{types::Uuid, PgPool};

use crate::error::AppError;
use crate::services::prize_service::EnabledPrize;
use crate::services::prize_cache::{snapshot as prize_snapshot, PrizeLite};
use redis::aio::ConnectionManager as RedisManager;
use crate::redis_scripts::{LUA_COOLDOWN_ONLY, LUA_COOLDOWN_AND_DECR};
use crate::redis_client::global_manager_from_env;

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

// Public API used by routes and tests: tries Redis+Lua, falls back to SQL-only if REDIS_URL missing/unavailable
pub async fn draw(pool: &PgPool, uid: Uuid) -> Result<DrawResult, AppError> {
    if let Ok(mut mgr) = global_manager_from_env().await {
        return draw_with_redis(pool, &mut mgr, uid).await;
    }
    draw_sql_only(pool, uid).await
}

// Redis path: requires a mutable connection manager
pub async fn draw_with_redis(pool: &PgPool, redis: &mut RedisManager, uid: Uuid) -> Result<DrawResult, AppError> {
    // 1) read enabled prizes from in-memory cache (fallback to DB if empty)
    let mut prizes_lite: Vec<PrizeLite> = prize_snapshot().await;
    if prizes_lite.is_empty() {
        let rows = sqlx::query_as::<_, (Uuid, String, i32)>(
            r#"SELECT id, name, probability FROM prizes WHERE is_enabled=true"#
        )
        .fetch_all(pool)
        .await?;
        prizes_lite = rows.into_iter().map(|(id,name,probability)| PrizeLite{ id, name, probability }).collect();
    }

    // 2) weighted selection
    let total_weight: i32 = prizes_lite.iter().map(|p| p.probability.max(0)).sum();
    let no_win_weight = (100 - total_weight).max(0);
    let roll = { let mut rng = rand::thread_rng(); rng.gen_range(1..=std::cmp::max(1, total_weight + no_win_weight)) };

    let mut acc = 0;
    let mut selected: Option<(Uuid, String)> = None;
    for p in &prizes_lite {
        acc += p.probability.max(0);
        if roll <= acc { selected = Some((p.id, p.name.clone())); break; }
    }

    // 3) if selected, atomically decr stock + set cooldown in Redis; otherwise set cooldown only
    let ttl = 60i64;
    let (won, prize_id, prize_name) = if let Some((pid, pname)) = selected {
        let stock_key = format!("lottery:stock:{}", pid);
        let sold_key = format!("lottery:sold:{}", pid);
        let r: i64 = LUA_COOLDOWN_AND_DECR
            .key(format!("lottery:cooldown:{}", uid))
            .key(stock_key)
            .key(sold_key)
            .arg(ttl)
            .invoke_async(redis)
            .await
            .unwrap_or(-1);
        if r == 0 { return Err(AppError::BadRequest("抽奖频率过高，请稍后再试")); }
        if r == 1 { (true, Some(pid), Some(pname)) } else { (false, None, None) }
    } else {
        let r: i64 = LUA_COOLDOWN_ONLY
            .key(format!("lottery:cooldown:{}", uid))
            .arg(ttl)
            .invoke_async(redis)
            .await
            .unwrap_or(0);
        if r == 0 { return Err(AppError::BadRequest("抽奖频率过高，请稍后再试")); }
        (false, None, None)
    };

    // 4) persist record asynchronously (fire-and-forget)
    if won || true {
        let pool = pool.clone();
        let prize_id_c = prize_id;
        let prize_name_c = prize_name.clone();
        tokio::spawn(async move {
            let _ = sqlx::query(
                r#"INSERT INTO lottery_records (id, user_id, prize_id, prize_name, created_at)
                    VALUES ($1,$2,$3,$4, now())"#
            )
            .bind(Uuid::new_v4())
            .bind(uid)
            .bind(prize_id_c)
            .bind(prize_name_c.as_deref())
            .execute(&pool)
            .await;

            let _ = sqlx::query("UPDATE users SET last_lottery_at=now(), updated_at=now() WHERE id=$1")
                .bind(uid)
                .execute(&pool)
                .await;
        });
    }

    Ok(DrawResult { won, prize_id, prize_name })
}

// SQL-only fallback (original implementation)
async fn draw_sql_only(pool: &PgPool, uid: Uuid) -> Result<DrawResult, AppError> {
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
        let row = sqlx::query(
            r#"UPDATE prizes SET remaining_count = remaining_count - 1, updated_at=now()
                WHERE id=$1 AND remaining_count>0 RETURNING remaining_count"#
        )
        .bind(pid)
        .fetch_optional(&mut *tx)
        .await?;
        if row.is_some() { (true, Some(pid), Some(pname)) } else { (false, None, None) }
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
