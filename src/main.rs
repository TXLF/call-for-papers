use call_for_papers::{api, config::Config, db};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "call_for_papers=debug,tower_http=debug,sqlx=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    tracing::info!("Starting server...");
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize database
    tracing::info!("Connecting to database...");
    db::ensure_database_exists(&config.database_url).await?;

    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Run migrations
    tracing::info!("Running database migrations...");
    match db::run_migrations(&pool).await {
        Ok(_) => tracing::info!("Migrations completed successfully"),
        Err(e) => {
            tracing::error!("Migration failed: {}", e);
            return Err(e.into());
        }
    }

    // Create uploads directory if it doesn't exist
    tracing::info!("Ensuring upload directory exists: {}", config.upload_dir);
    std::fs::create_dir_all(&config.upload_dir)?;

    // Create API router with database pool and config
    let app = api::create_router(pool, config.clone());

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
