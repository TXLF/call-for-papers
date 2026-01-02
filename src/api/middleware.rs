use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};

use crate::{
    api::AppState,
    handlers::auth::verify_token,
    models::{auth::ErrorResponse, User},
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Missing authorization header")),
            )
        })?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid authorization header format")),
        )
    })?;

    let user = verify_token(token, &state.db, &state.config.jwt_secret)
        .await
        .map_err(|e| {
            tracing::error!("Token verification failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Invalid or expired token")),
            )
        })?;

    // Add user to request extensions so handlers can access it
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

pub async fn organizer_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let user = req.extensions().get::<User>().ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Authentication required")),
        )
    })?;

    if !user.is_organizer {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("Organizer access required")),
        ));
    }

    Ok(next.run(req).await)
}
