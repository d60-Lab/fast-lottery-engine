use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::OnceCell;
use sqlx::{PgPool, types::Uuid};

#[derive(Clone, Debug)]
pub struct PrizeLite { pub id: Uuid, pub name: String, pub probability: i32 }

static CACHE: OnceCell<Arc<RwLock<Vec<PrizeLite>>>> = OnceCell::const_new();

pub async fn get_cache() -> Arc<RwLock<Vec<PrizeLite>>> {
    CACHE.get_or_init(|| async { Arc::new(RwLock::new(Vec::new())) }).await.clone()
}

pub fn spawn_refresh(pool: PgPool) {
    let cache_fut = get_cache();
    tokio::spawn(async move {
        let cache = cache_fut.await;
        let mut tick = tokio::time::interval(Duration::from_millis(800));
        loop {
            tick.tick().await;
            let rows: Result<Vec<(Uuid, String, i32)>, _> = sqlx::query_as(
                "SELECT id, name, probability FROM prizes WHERE is_enabled=true"
            )
            .fetch_all(&pool)
            .await
            .map(|v: Vec<(Uuid, String, i32)>| v);
            if let Ok(v) = rows {
                let list: Vec<PrizeLite> = v.into_iter().map(|(id, name, probability)| PrizeLite{ id, name, probability }).collect();
                *cache.write().await = list;
            }
        }
    });
}

pub async fn snapshot() -> Vec<PrizeLite> {
    get_cache().await.read().await.clone()
}
