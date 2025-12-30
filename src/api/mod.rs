pub mod middleware;

use axum::{
    extract::State,
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use tower_http::services::{ServeDir, ServeFile};

use crate::{config::Config, handlers, services::EmailService};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub email_service: EmailService,
    pub claude_service: crate::services::ClaudeService,
}

pub fn create_router(db: PgPool, config: Config) -> Router {
    let upload_dir = config.upload_dir.clone();
    let email_service = EmailService::new(config.clone(), db.clone());
    let claude_service = crate::services::ClaudeService::new(&config);
    let state = AppState {
        db,
        config,
        email_service,
        claude_service,
    };

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
        // Conference routes (organizer only for CUD operations)
        .route("/conferences", post(handlers::create_conference))
        .route("/conferences/:id", put(handlers::update_conference))
        .route("/conferences/:id", delete(handlers::delete_conference))
        // Track routes (organizer only for CUD operations)
        .route("/tracks", post(handlers::create_track))
        .route("/tracks/:id", put(handlers::update_track))
        .route("/tracks/:id", delete(handlers::delete_track))
        // Schedule slot routes (organizer only for CUD operations)
        .route("/schedule-slots", post(handlers::create_schedule_slot))
        .route("/schedule-slots/:id", put(handlers::update_schedule_slot))
        .route("/schedule-slots/:id", delete(handlers::delete_schedule_slot))
        .route("/schedule-slots/:id/assign", put(handlers::assign_talk_to_slot))
        .route("/schedule-slots/:id/assign", delete(handlers::unassign_talk_from_slot))
        // Email template routes (organizer only)
        .route("/email-templates", get(handlers::list_email_templates))
        .route("/email-templates/:id", get(handlers::get_email_template))
        .route("/email-templates", post(handlers::create_email_template))
        .route("/email-templates/:id", put(handlers::update_email_template))
        .route("/email-templates/:id", delete(handlers::delete_email_template))
        // Bulk email route (organizer only)
        .route("/bulk-email", post(handlers::send_bulk_email))
        // Export route (organizer only)
        .route("/export/talks", get(handlers::export_talks))
        // AI tagging routes (organizer only)
        .route("/ai/auto-tag", get(handlers::auto_tag_with_claude))
        .route("/ai/create-labels", post(handlers::create_ai_labels))
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
        // OAuth routes
        .route("/auth/google", get(handlers::google_authorize))
        .route("/auth/google/callback", get(handlers::google_callback))
        .route("/auth/github", get(handlers::github_authorize))
        .route("/auth/github/callback", get(handlers::github_callback))
        .route("/auth/apple", get(handlers::apple_authorize))
        .route("/auth/apple/callback", get(handlers::apple_callback))
        .route("/auth/facebook", get(handlers::facebook_authorize))
        .route("/auth/facebook/callback", get(handlers::facebook_callback))
        .route("/auth/linkedin", get(handlers::linkedin_authorize))
        .route("/auth/linkedin/callback", get(handlers::linkedin_callback))
        // Public label routes
        .route("/labels", get(handlers::list_labels))
        // Public conference routes (read-only)
        .route("/conferences", get(handlers::list_conferences))
        .route("/conferences/active", get(handlers::get_active_conference))
        .route("/conferences/:id", get(handlers::get_conference))
        // Public track routes (read-only)
        .route("/tracks", get(handlers::list_tracks))
        .route("/tracks/:id", get(handlers::get_track))
        // Public schedule slot routes (read-only)
        .route("/schedule-slots", get(handlers::list_schedule_slots))
        .route("/schedule-slots/:id", get(handlers::get_schedule_slot))
        // Public schedule view (with talk details)
        .route("/schedule", get(handlers::get_public_schedule))
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
