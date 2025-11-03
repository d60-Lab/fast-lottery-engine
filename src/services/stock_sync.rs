use std::time::Duration;
use sqlx::{PgPool, types::Uuid};
use redis::aio::ConnectionManager as RedisManager;

pub fn spawn_redis_delta_flusher(pool: PgPool, redis: std::sync::Arc<RedisManager>) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            // read enabled prize ids
            let ids: Vec<Uuid> = match sqlx::query_scalar(
                "SELECT id FROM prizes WHERE is_enabled=true"
            ).fetch_all(&pool).await {
                Ok(v) => v,
                Err(_) => continue,
            };
            let mut conn = (*redis).clone();
            for pid in ids {
                let key = format!("lottery:sold:{}", pid);
                // atomic get and delete counter; if not supported, fall back to GET then DEL best-effort
                let delta_opt: redis::RedisResult<Option<i64>> = redis::cmd("GETDEL")
                    .arg(&key)
                    .query_async(&mut conn)
                    .await;
                let delta = delta_opt.unwrap_or(None).unwrap_or(0);
                if delta > 0 {
                    let _ = sqlx::query(
                        "UPDATE prizes SET remaining_count = GREATEST(0, remaining_count - $1), updated_at=now() WHERE id=$2"
                    )
                    .bind(delta)
                    .bind(pid)
                    .execute(&pool)
                    .await;
                }
            }
        }
    });
}
