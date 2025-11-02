use chrono::{Duration, Utc};
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, types::Uuid, Pool, Postgres};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let desired_stock: i64 = std::env::var("PREP_STOCK").ok().and_then(|s| s.parse().ok()).unwrap_or(500_000);
    let probability: i32 = std::env::var("PREP_PROBABILITY").ok().and_then(|s| s.parse().ok()).unwrap_or(100);

    let pool: Pool<Postgres> = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    // ensure an ongoing activity
    let act_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM activities WHERE status='ongoing' AND start_time<=now() AND end_time>=now() LIMIT 1")
        .fetch_optional(&pool)
        .await?;
    let act_id = match act_id {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4();
            let start = Utc::now() - Duration::days(1);
            let end = Utc::now() + Duration::days(7);
            sqlx::query(r#"INSERT INTO activities (id, name, description, start_time, end_time, status, created_at, updated_at)
                VALUES ($1,$2,$3,$4,$5,$6, now(), now())"#)
                .bind(id)
                .bind("Bench Activity")
                .bind(Option::<String>::None)
                .bind(start)
                .bind(end)
                .bind("ongoing")
                .execute(&pool)
                .await?;
            id
        }
    };

    // ensure a big stock prize named 'PerfPrize'
    let prize_exists: Option<(Uuid, i64, i64)> = sqlx::query_as(
        "SELECT id, total_count, remaining_count FROM prizes WHERE name='PerfPrize' AND activity_id=$1 LIMIT 1"
    )
    .bind(act_id)
    .fetch_optional(&pool)
    .await?;

    if let Some((pid, total, remain)) = prize_exists {
        let new_total = total.max(desired_stock);
        let new_remain = remain.max(desired_stock);
        sqlx::query("UPDATE prizes SET total_count=$1, remaining_count=$2, probability=$3, is_enabled=true, updated_at=now() WHERE id=$4")
            .bind(new_total)
            .bind(new_remain)
            .bind(probability)
            .bind(pid)
            .execute(&pool)
            .await?;
    } else {
        let pid = Uuid::new_v4();
        sqlx::query(r#"INSERT INTO prizes (id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$5,$6,true, now(), now())"#)
            .bind(pid)
            .bind(act_id)
            .bind("PerfPrize")
            .bind(Option::<String>::None)
            .bind(desired_stock)
            .bind(probability)
            .execute(&pool)
            .await?;
    }

    println!("prepared inventory: activity={} stock={} prob={}", act_id, desired_stock, probability);
    Ok(())
}
