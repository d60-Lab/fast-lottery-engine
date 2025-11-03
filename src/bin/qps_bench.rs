use std::sync::{atomic::{AtomicU64, Ordering}, Arc};
use std::time::{Duration, Instant};

use reqwest::Client;
use dotenvy::dotenv;
use serde_json::json;
use tokio::task::JoinSet;

fn env_usize(name: &str, default_: usize) -> usize {
    std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(default_)
}

fn env_string(name: &str, default_: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default_.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load .env for ADMIN_USERNAME/ADMIN_PASSWORD, DATABASE_URL etc.
    let _ = dotenv();
    // Config via env: BENCH_URL, BENCH_OPS, BENCH_CONC
    let base = env_string("BENCH_URL", "http://127.0.0.1:8080");
    let ops = env_usize("BENCH_OPS", 10000);
    let conc = env_usize("BENCH_CONC", 256);

    println!("bench target: {} | ops: {} | concurrency: {}", base, ops, conc);

    let client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(100)
        .tcp_keepalive(Duration::from_secs(30))
        .build()?;

    // 1) prepare tokens
    let mut tokens = Vec::with_capacity(ops);
    let start_prep = Instant::now();
    if std::env::var("BENCH_FAST").ok().as_deref() == Some("1") {
        // use admin fast path to mint tokens w/o argon2
        let admin_user = env_string("ADMIN_USERNAME", "admin");
        let admin_pass = env_string("ADMIN_PASSWORD", "admin");
        let login_resp = client
            .post(format!("{}/admin/api/login", base))
            .json(&json!({"username": admin_user, "password": admin_pass}))
            .send()
            .await;
        if let Ok(rsp) = login_resp {
            let status = rsp.status();
            let body = rsp.text().await.unwrap_or_default();
            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(admin_tok) = j.get("token").and_then(|t| t.as_str()) {
                    let mint_resp = client
                        .post(format!("{}/admin/api/bench/mint-tokens", base))
                        .header("authorization", format!("Bearer {}", admin_tok))
                        .json(&json!({"count": ops, "prefix":"bench_user_"}))
                        .send()
                        .await;
                    if let Ok(minted) = mint_resp {
                        let minted_status = minted.status();
                        let body = minted.text().await.unwrap_or_default();
                        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(arr) = j.get("tokens").and_then(|a| a.as_array()) {
                                tokens = arr
                                    .iter()
                                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                    .collect();
                            }
                        } else {
                            eprintln!("[warn] mint-tokens non-json (status={}): {}", minted_status, &body[..body.len().min(200)]);
                        }
                    } else {
                        eprintln!("[warn] mint-tokens request failed");
                    }
                } else {
                    eprintln!("[warn] admin login ok but no token in body: {}", j);
                }
            } else {
                eprintln!("[warn] admin login non-json (status={}): {}", status, &body[..body.len().min(200)]);
            }
        } else {
            eprintln!("[warn] admin login request failed");
        }
        // if fast path failed, we will fallback below
    } else {
        // fallback path handled below
    }

    if tokens.is_empty() {
        // fallback: register per-user (slow due to argon2)
        let mut js = JoinSet::new();
        let url_register = format!("{}/api/auth/register", base);
        let url_login = format!("{}/api/auth/login", base);
        let mut submitted = 0usize;
        while submitted < ops {
            while js.len() < conc && submitted < ops {
                let client = client.clone();
                let url_register = url_register.clone();
                let uname = format!("bench_user_{}", submitted);
                let pwd = "bench_pass_123".to_string();
                js.spawn(async move {
                    let body = json!({"username": uname, "password": pwd});
                    let res = client.post(&url_register).json(&body).send().await;
                    match res {
                        Ok(r) => {
                            let v: serde_json::Value = r.json().await.unwrap_or(json!({}));
                            v.get("token").and_then(|t| t.as_str()).map(|s| s.to_string())
                        }
                        Err(_) => None,
                    }
                });
                submitted += 1;
            }
            if let Some(r) = js.join_next().await { if let Ok(Some(tok)) = r { tokens.push(tok); } }
        }
        while let Some(r) = js.join_next().await { if let Ok(Some(tok)) = r { tokens.push(tok); } }
        // try login for any missing
        while tokens.len() < ops {
            let missing = ops - tokens.len();
            let batch = missing.min(conc);
            let mut started = 0usize;
            while started < batch {
                let client = client.clone();
                let url_login = url_login.clone();
                let idx = tokens.len() + started;
                let uname = format!("bench_user_{}", idx);
                let pwd = "bench_pass_123".to_string();
                js.spawn(async move {
                    let body = json!({"username": uname, "password": pwd});
                    match client.post(&url_login).json(&body).send().await {
                        Ok(r) => {
                            let v: serde_json::Value = r.json().await.unwrap_or(json!({}));
                            v.get("token").and_then(|t| t.as_str()).map(|s| s.to_string())
                        }
                        Err(_) => None,
                    }
                });
                started += 1;
            }
            for _ in 0..batch { if let Some(r) = js.join_next().await { if let Ok(Some(tok)) = r { tokens.push(tok); } } }
        }
    }
    if tokens.len() < ops {
        eprintln!("[warn] only prepared {} tokens (< ops={}), will proceed with what we have", tokens.len(), ops);
    }
    println!("prepared {} tokens in {:?}", tokens.len(), start_prep.elapsed());

    // 2) run draw bench: one draw per token (避免频率限制影响)
    let url_draw = format!("{}/api/lottery/draw", base);
    let cnt = Arc::new(AtomicU64::new(0));
    let mut lat = Vec::with_capacity(ops);
    lat.resize(ops, 0u128);

    let t0 = Instant::now();
    let mut idx = 0usize;
    let mut inflight = JoinSet::new();
    while idx < tokens.len() || !inflight.is_empty() {
        while inflight.len() < conc && idx < tokens.len() {
            let client = client.clone();
            let url_draw = url_draw.clone();
            let token = tokens[idx].clone();
            let slot = idx;
            inflight.spawn(async move {
                let s = Instant::now();
                let res = client.post(&url_draw).header("authorization", format!("Bearer {}", token)).send().await;
                let _ = res; // even on error we count attempt
                s.elapsed().as_micros()
            });
            idx += 1;
        }
        if let Some(r) = inflight.join_next().await {
            if let Ok(us) = r { let i = cnt.fetch_add(1, Ordering::Relaxed) as usize; if i < lat.len() { lat[i] = us; } }
        }
    }
    let elapsed = t0.elapsed();
    let completed = cnt.load(Ordering::Relaxed) as usize;
    lat.truncate(completed);
    lat.sort_unstable();

    let q_ms = |p: f64| -> f64 {
        if lat.is_empty() { return 0.0; }
        let idx = ((lat.len() as f64) * p).ceil() as usize - 1;
        (lat[idx.min(lat.len()-1)] as f64) / 1000.0
    };
    let avg_ms: f64 = if !lat.is_empty() { (lat.iter().sum::<u128>() as f64 / lat.len() as f64) / 1000.0 } else { 0.0 };
    let qps = (completed as f64) / elapsed.as_secs_f64();
    println!("draw bench completed: ops={} done={} time={:?} qps={:.2} avg={:.2}ms p50={:.2}ms p95={:.2}ms p99={:.2}ms",
        ops, completed, elapsed, qps, avg_ms, q_ms(0.50), q_ms(0.95), q_ms(0.99));

    Ok(())
}
