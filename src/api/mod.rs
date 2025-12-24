use axum::{
    extract::State,
    routing::get,
    Router,
};
use sqlx::PgPool;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn create_router(db: PgPool) -> Router {
    let state = AppState { db };

    // API routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .with_state(state);

    // Check if frontend dist directory exists
    let frontend_path = std::path::Path::new("frontend/dist");

    if frontend_path.exists() {
        // Serve static files with fallback to index.html for SPA routing
        let serve_dir = ServeDir::new("frontend/dist")
            .not_found_service(ServeFile::new("frontend/dist/index.html"));

        Router::new()
            .nest("/api", api_routes)
            .fallback_service(serve_dir)
    } else {
        tracing::warn!("Frontend dist directory not found, serving API only");
        api_routes
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn health_check_db(State(state): State<AppState>) -> Result<&'static str, String> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok("OK")
}
