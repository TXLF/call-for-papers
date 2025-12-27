pub mod middleware;

use axum::{
    extract::State,
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use tower_http::services::{ServeDir, ServeFile};

use crate::{config::Config, handlers};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

pub fn create_router(db: PgPool, config: Config) -> Router {
    let state = AppState { db, config };

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        // Talk routes
        .route("/talks", post(handlers::create_talk))
        .route("/talks/mine", get(handlers::get_my_talks))
        .route("/talks/:id", get(handlers::get_talk))
        .route("/talks/:id", put(handlers::update_talk))
        .route("/talks/:id", delete(handlers::delete_talk))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Public API routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        // Authentication routes
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .merge(protected_routes)
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
