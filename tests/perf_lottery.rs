use std::{path::Path, time::Instant};

use fast_lottery_engine::services::{lottery_service, user_service};
use sqlx::types::Uuid;
use sqlx_db_tester::TestPg;
use tokio::task::JoinSet;

fn base_url() -> String {
    std::env::var("TEST_PG_URL").expect("set TEST_PG_URL for tests")
}

fn env_usize(name: &str, default_: usize) -> usize {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default_)
}

// Ignored by default: run with `cargo test --test perf_lottery -- --ignored`
#[tokio::test]
#[ignore]
async fn perf_draw_concurrency() {
    let ops = env_usize("PERF_OPS", 500);
    let conc = env_usize("PERF_CONCURRENCY", 50);

    let tdb = TestPg::new(base_url(), Path::new("./migrations"));
    let pool = tdb.get_pool().await;

    // Ensure there is plenty of stock for performance run: insert a big prize under any activity
    let act_id: Uuid = sqlx::query_scalar("SELECT id FROM activities LIMIT 1").fetch_one(&pool).await.unwrap();
    let prize_id = Uuid::new_v4();
    let total = (ops as i64) + 100;
    sqlx::query(
        r#"INSERT INTO prizes (id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at)
           VALUES ($1,$2,$3,$4,$5,$5,$6,true, now(), now())"#
    )
    .bind(prize_id)
    .bind(act_id)
    .bind("PerfPrize")
    .bind(Option::<String>::None)
    .bind(total)
    .bind(100i32)
    .execute(&pool)
    .await
    .unwrap();

    // Create one-time users to bypass frequency limit
    let mut users = Vec::with_capacity(ops);
    for i in 0..ops {
        let uid = Uuid::new_v4();
        user_service::create_user(&pool, uid, &format!("perf_user_{}", i), "HASH", &None)
            .await
            .unwrap();
        users.push(uid);
    }

    let start = Instant::now();
    let mut js = JoinSet::new();
    let mut completed: usize = 0;
    let mut durations: Vec<u128> = Vec::with_capacity(ops);

    let mut it = users.into_iter();
    loop {
        while js.len() < conc {
            match it.next() {
                Some(uid) => {
                    let pool_cloned = pool.clone();
                    js.spawn(async move {
                        let t0 = Instant::now();
                        let _ = lottery_service::draw(&pool_cloned, uid).await;
                        t0.elapsed().as_micros()
                    });
                }
                None => break,
            }
        }
        if js.is_empty() { break; }
        if let Some(res) = js.join_next().await { match res { Ok(us) => { completed += 1; durations.push(us); }, Err(_) => {} } }
    }
    let total_elapsed = start.elapsed();

    assert_eq!(completed, ops, "not all operations finished");

    durations.sort_unstable();
    let p = |q: f64| -> u128 {
        let idx = ((durations.len() as f64) * q).ceil() as usize - 1;
        durations[idx.min(durations.len()-1)]
    };
    eprintln!(
        "perf draw: ops={}, concurrency={}, total={:?}, avg={}us, p50={}us, p95={}us, p99={}us",
        ops,
        conc,
        total_elapsed,
        (durations.iter().sum::<u128>() as f64 / durations.len() as f64) as u128,
        p(0.50),
        p(0.95),
        p(0.99)
    );
}
