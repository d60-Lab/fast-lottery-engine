use std::path::Path;

use fast_lottery_engine::services::{lottery_service, user_service};
use sqlx::types::Uuid;
use sqlx_db_tester::TestPg;

fn base_url() -> String {
    std::env::var("TEST_PG_URL").expect("set TEST_PG_URL for tests")
}

#[tokio::test]
async fn draw_flow_with_new_user() {
    let _ = dotenvy::dotenv();
    let tdb = TestPg::new(base_url(), Path::new("./migrations"));
    let pool = tdb.get_pool().await;

    // create a user
    let uid = Uuid::new_v4();
    user_service::create_user(&pool, uid, "tester", "HASH", &None).await.unwrap();

    // draw once; result should be either won or not, but no error
    let res = lottery_service::draw(&pool, uid).await.unwrap();
    assert!(res.won || (!res.won && res.prize_id.is_none()));
}
