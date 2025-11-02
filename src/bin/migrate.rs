use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    // run all migrations under ./migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("migrations applied successfully");
    Ok(())
}
