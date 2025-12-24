use sqlx::postgres::PgPoolOptions;
use sqlx::{migrate::MigrateDatabase, PgPool, Postgres};

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn ensure_database_exists(database_url: &str) -> Result<(), sqlx::Error> {
    if !Postgres::database_exists(database_url).await? {
        tracing::info!("Creating database...");
        Postgres::create_database(database_url).await?;
        tracing::info!("Database created successfully");
    }
    Ok(())
}
