use axum::{
    extract::State,
    routing::get,
    Router,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn create_router(db: PgPool) -> Router {
    let state = AppState { db };

    Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .with_state(state)
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
