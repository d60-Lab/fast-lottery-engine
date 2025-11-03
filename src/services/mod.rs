use sqlx::PgPool;

pub mod user_service;
pub mod activity_service;
pub mod prize_service;
pub mod lottery_service;
pub mod stock_sync;
pub mod prize_cache;

pub type Db = PgPool;
