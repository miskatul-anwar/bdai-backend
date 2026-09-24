use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Connecting to PostgreSQL/Supabase database...");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect(database_url)
        .await?;

    info!("Database connection established successfully.");
    Ok(pool)
}

/// Automatically ensures tables exist in public schema (fallback initialization)
pub async fn run_migrations_if_needed(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Verifying database schema...");

    // Check if tables already exist
    let table_exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' AND table_name = 'users'
        )",
    )
    .fetch_one(pool)
    .await?;

    if !table_exists.0 {
        info!("Running initial BDAI schema creation on Supabase...");
        let schema_sql = include_str!("../sql/schema.sql");
        sqlx::raw_sql(schema_sql).execute(pool).await?;

        info!("Seeding initial BDAI demo personnel, users, notices, and objectives...");
        let seed_sql = include_str!("../sql/seed.sql");
        sqlx::raw_sql(seed_sql).execute(pool).await?;

        info!("Initial database provisioning completed successfully.");
    } else {
        info!("Database schema is up to date.");
    }

    Ok(())
}
