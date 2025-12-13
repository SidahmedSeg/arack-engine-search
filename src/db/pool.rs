use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

/// Create a PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    info!("Creating database connection pool...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    info!("Database connection pool created successfully");
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    info!("Database migrations completed successfully");
    Ok(())
}
