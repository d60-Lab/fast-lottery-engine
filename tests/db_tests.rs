use std::path::Path;

use sqlx::{Row, types::Uuid};
use sqlx_db_tester::TestPg;

// Use TEST_PG_URL to override base connection if needed; default assumes local dev
fn base_url() -> String {
    std::env::var("TEST_PG_URL").expect("set TEST_PG_URL for tests")
}

#[tokio::test]
async fn migrations_and_seed_exist() {
    let tdb = TestPg::new(base_url(), Path::new("./migrations"));
    let pool = tdb.get_pool().await;

    // activities present (from seed 0002)
    let act_cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activities").fetch_one(&pool).await.unwrap();
    assert!(act_cnt >= 1, "expected at least one activity from seed");

    // prizes present (from seed 0002)
    let prize_cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM prizes").fetch_one(&pool).await.unwrap();
    assert!(prize_cnt >= 3, "expected at least three prizes from seed");

    // ensure there's an ongoing activity
    let ongoing_cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activities WHERE status = 'ongoing'")
        .fetch_one(&pool).await.unwrap();
    assert!(ongoing_cnt >= 1, "expected at least one ongoing activity");
}

#[tokio::test]
async fn prize_inventory_decrements_atomically() {
    let tdb = TestPg::new(base_url(), Path::new("./migrations"));
    let pool = tdb.get_pool().await;

    // pick any enabled prize with stock
    #[derive(sqlx::FromRow, Debug)]
    struct P { id: Uuid, remaining_count: i64 }
    let prize: P = sqlx::query_as::<_, P>(
        "SELECT id, remaining_count FROM prizes WHERE is_enabled=true AND remaining_count>0 LIMIT 1"
    ).fetch_one(&pool).await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    // decrement atomically
    let row = sqlx::query(
        r#"UPDATE prizes SET remaining_count = remaining_count - 1, updated_at=now()
            WHERE id=$1 AND remaining_count>0 RETURNING remaining_count"#
    )
    .bind(prize.id)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let new_remaining: i64 = row.get::<i64, _>("remaining_count");
    assert_eq!(new_remaining, prize.remaining_count - 1);
    tx.commit().await.unwrap();
}
