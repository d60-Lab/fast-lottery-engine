use redis::aio::ConnectionManager;
use tokio::sync::OnceCell;

static REDIS_MGR: OnceCell<ConnectionManager> = OnceCell::const_new();

pub async fn connect_manager(redis_url: &str) -> anyhow::Result<ConnectionManager> {
    let client = redis::Client::open(redis_url)?;
    let mgr = client.get_connection_manager().await?;
    Ok(mgr)
}

pub async fn global_manager_from_env() -> anyhow::Result<ConnectionManager> {
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mgr = REDIS_MGR
        .get_or_try_init(|| async {
            let client = redis::Client::open(url.clone())?;
            let mgr = client.get_connection_manager().await?;
            Ok::<ConnectionManager, redis::RedisError>(mgr)
        })
        .await?;
    Ok(mgr.clone())
}
