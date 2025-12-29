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
    let upload_dir = config.upload_dir.clone();
    let state = AppState { db, config };

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        // Talk routes
        .route("/talks", post(handlers::create_talk))
        .route("/talks/mine", get(handlers::get_my_talks))
        .route("/talks/:id", get(handlers::get_talk))
        .route("/talks/:id", put(handlers::update_talk))
        .route("/talks/:id", delete(handlers::delete_talk))
        .route("/talks/:id/upload-slides", post(handlers::upload_slides))
        .route("/talks/:id/respond", post(handlers::respond_to_talk))
        // Talk-label routes
        .route("/talks/:id/labels", get(handlers::get_talk_labels))
        .route("/talks/:id/labels", post(handlers::add_labels_to_talk))
        .route("/talks/:id/labels/:label_id", delete(handlers::remove_label_from_talk))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Organizer-only routes
    let organizer_routes = Router::new()
        .route("/dashboard/stats", get(handlers::get_dashboard_stats))
        .route("/talks", get(handlers::list_all_talks))
        .route("/talks/:id/state", put(handlers::change_talk_state))
        .route("/labels", post(handlers::create_label))
        .route("/labels/:id", put(handlers::update_label))
        .route("/labels/:id", delete(handlers::delete_label))
        // Rating routes
        .route("/talks/:id/rate", post(handlers::create_or_update_rating))
        .route("/talks/:id/ratings", get(handlers::get_talk_ratings))
        .route("/talks/:id/rate/mine", get(handlers::get_my_rating))
        .route("/talks/:id/rate", delete(handlers::delete_rating))
        .route("/ratings/statistics", get(handlers::get_ratings_statistics))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .layer(axum_middleware::from_fn(middleware::organizer_middleware));

    // Public API routes
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        // Authentication routes
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        // Public label routes
        .route("/labels", get(handlers::list_labels))
        .merge(protected_routes)
        .merge(organizer_routes)
        .with_state(state);

    // Serve uploaded files
    let uploads_service = ServeDir::new(&upload_dir);

    // Check if frontend dist directory exists
    let frontend_path = std::path::Path::new("frontend/dist");

    if frontend_path.exists() {
        // Serve static files with fallback to index.html for SPA routing
        let serve_dir = ServeDir::new("frontend/dist")
            .not_found_service(ServeFile::new("frontend/dist/index.html"));

        Router::new()
            .nest("/api", api_routes)
            .nest_service("/uploads", uploads_service)
            .fallback_service(serve_dir)
    } else {
        tracing::warn!("Frontend dist directory not found, serving API only");
        Router::new()
            .nest("/api", api_routes)
            .nest_service("/uploads", uploads_service)
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
