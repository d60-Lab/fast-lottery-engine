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
use rand::Rng;
use serde::Serialize;
use sqlx::types::Uuid;

#[derive(serde::Serialize)]
pub struct DrawResult {
    pub won: bool,
    pub prize_id: Option<Uuid>,
    pub prize_name: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct PrizeRow {
    id: Uuid,
    activity_id: Uuid,
    name: String,
    description: Option<String>,
    total_count: i64,
    remaining_count: i64,
    probability: i32,
    is_enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub async fn list_prizes(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let prizes: Vec<PrizeRow> = sqlx::query_as(
        r#"SELECT id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at
           FROM prizes WHERE is_enabled=true AND remaining_count>0 ORDER BY updated_at DESC"#
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(serde_json::json!({"prizes": prizes})))
}

#[axum::debug_handler]
pub async fn draw(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<DrawResult>> {
    let claims = verify_jwt(&state.cfg, bearer.token())?;
    let uid = Uuid::parse_str(&claims.uid).map_err(|_| AppError::Unauthorized)?;

    let mut tx = state.pool.begin().await?;

    // 频率限制：60秒一次
    let last: Option<DateTime<Utc>> =
        sqlx::query_scalar("SELECT last_lottery_at FROM users WHERE id=$1 FOR UPDATE")
            .bind(uid)
            .fetch_one(&mut *tx)
            .await?;
    if let Some(last) = last {
        let seconds = (chrono::Utc::now() - last).num_seconds();
        if seconds < 60 {
            return Err(AppError::BadRequest("抽奖频率过高，请稍后再试"));
        }
    }

    // 获取可用奖品
    #[derive(sqlx::FromRow)]
    struct P {
        id: Uuid,
        name: String,
        remaining_count: i64,
        probability: i32,
    }
    let prizes: Vec<P> = sqlx::query_as(
        r#"SELECT id, name, remaining_count, probability FROM prizes
        WHERE is_enabled=true AND remaining_count>0"#,
    )
    .fetch_all(&mut *tx)
    .await?;

    // 计算权重含"未中奖"
    let total_weight: i32 = prizes.iter().map(|p| p.probability.max(0)).sum();
    let no_win_weight = (100 - total_weight).max(0);
    let roll = {
        let mut rng = rand::thread_rng();
        rng.gen_range(1..=std::cmp::max(1, total_weight + no_win_weight))
    };

    let mut acc = 0;
    let mut selected: Option<(Uuid, String)> = None;
    for p in &prizes {
        acc += p.probability.max(0);
        if roll <= acc {
            selected = Some((p.id, p.name.clone()));
            break;
        }
    }

    let (won, prize_id, prize_name) = if let Some((pid, pname)) = selected {
        // 尝试扣减库存
        let updated = sqlx::query(
            r#"UPDATE prizes SET remaining_count = remaining_count - 1, updated_at=now()
                WHERE id=$1 AND remaining_count>0 RETURNING remaining_count"#,
        )
        .bind(pid)
        .fetch_optional(&mut *tx)
        .await?;
        if updated.is_some() {
            (true, Some(pid), Some(pname))
        } else {
            (false, None, None)
        }
    } else {
        (false, None, None)
    };

    // 记录抽奖结果
    sqlx::query(
        r#"INSERT INTO lottery_records (id, user_id, prize_id, prize_name, created_at)
            VALUES ($1,$2,$3,$4, now())"#,
    )
    .bind(Uuid::new_v4())
    .bind(uid)
    .bind(prize_id)
    .bind(prize_name.as_deref())
    .execute(&mut *tx)
    .await?;

    // 更新用户last_lottery_at
    sqlx::query("UPDATE users SET last_lottery_at=now(), updated_at=now() WHERE id=$1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(DrawResult {
        won,
        prize_id,
        prize_name,
    }))
}

pub async fn global_history(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    #[derive(Serialize, sqlx::FromRow)]
    struct RecRow {
        id: Uuid,
        user_id: Uuid,
        prize_id: Option<Uuid>,
        prize_name: Option<String>,
        created_at: DateTime<Utc>,
    }
    let rows: Vec<RecRow> = sqlx::query_as(
        r#"SELECT id, user_id, prize_id, prize_name, created_at FROM lottery_records
           ORDER BY created_at DESC LIMIT 200"#,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(serde_json::json!({"records": rows})))
}
