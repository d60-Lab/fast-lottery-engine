use std::path::Path;

use axum::{body::Body, http::{Request, StatusCode}, Router};
use http_body_util::BodyExt; // collect()
use serde_json::json;
use tower::util::ServiceExt; // oneshot

use fast_lottery_engine::{config::Config, routes::{admin_routes, auth_routes, lottery_routes, user_routes}};
use sqlx_db_tester::TestPg;

fn base_url() -> String {
    std::env::var("TEST_PG_URL").expect("set TEST_PG_URL for tests")
}

fn test_cfg(db_url: String) -> Config {
    Config {
        database_url: db_url,
        jwt_secret: "test_secret".to_string(),
        server_addr: "127.0.0.1:0".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "admin".to_string(),
    }
}

#[tokio::test]
async fn full_flow_register_login_draw() {
    let tdb = TestPg::new(base_url(), Path::new("./migrations"));
    let pool = tdb.get_pool().await;
    let cfg = test_cfg("unused".to_string());

    let app: Router = Router::new()
        .merge(auth_routes(&pool, &cfg))
        .merge(user_routes(&pool, &cfg))
        .merge(lottery_routes(&pool, &cfg))
        .merge(admin_routes(&pool, &cfg));

    // register
    let body = json!({"username":"user1","password":"secret123"}).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type","application/json")
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = v.get("token").and_then(|x| x.as_str()).unwrap().to_string();

    // list prizes
    let req = Request::builder().method("GET").uri("/api/lottery/prizes").body(Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // draw
    let req = Request::builder()
        .method("POST")
        .uri("/api/lottery/draw")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
