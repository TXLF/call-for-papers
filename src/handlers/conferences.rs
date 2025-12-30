use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, Conference, ConferenceResponse, CreateConferenceRequest,
        UpdateConferenceRequest,
    },
};

/// List all conferences (public endpoint)
pub async fn list_conferences(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConferenceResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let conferences = sqlx::query_as::<_, Conference>(
        r#"
        SELECT * FROM conferences
        ORDER BY start_date DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching conferences: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch conferences")),
        )
    })?;

    let responses: Vec<ConferenceResponse> = conferences
        .into_iter()
        .map(ConferenceResponse::from)
        .collect();
    Ok(Json(responses))
}

/// Get the active conference (public endpoint)
pub async fn get_active_conference(
    State(state): State<AppState>,
) -> Result<Json<ConferenceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let conference = sqlx::query_as::<_, Conference>(
        r#"
        SELECT * FROM conferences
        WHERE is_active = true
        ORDER BY start_date DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching active conference: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch active conference")),
        )
    })?;

    let conference = conference.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("No active conference found")),
        )
    })?;

    Ok(Json(ConferenceResponse::from(conference)))
}

/// Get a single conference by ID (public endpoint)
pub async fn get_conference(
    State(state): State<AppState>,
    Path(conference_id): Path<Uuid>,
) -> Result<Json<ConferenceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let conference = sqlx::query_as::<_, Conference>(
        r#"
        SELECT * FROM conferences
        WHERE id = $1
        "#,
    )
    .bind(conference_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching conference: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch conference")),
        )
    })?;

    let conference = conference.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Conference not found")),
        )
    })?;

    Ok(Json(ConferenceResponse::from(conference)))
}

/// Create a new conference (organizer only)
pub async fn create_conference(
    State(state): State<AppState>,
    Json(payload): Json<CreateConferenceRequest>,
) -> Result<(StatusCode, Json<ConferenceResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate name
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Conference name is required")),
        ));
    }

    // Validate dates
    if payload.end_date < payload.start_date {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("End date must be after start date")),
        ));
    }

    // Create the conference
    let conference = sqlx::query_as::<_, Conference>(
        r#"
        INSERT INTO conferences (name, description, start_date, end_date, location, is_active)
        VALUES ($1, $2, $3, $4, $5, true)
        RETURNING *
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.description.as_ref().map(|s| s.trim()))
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.location.as_ref().map(|s| s.trim()))
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating conference: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create conference")),
        )
    })?;

    tracing::info!("Conference created: {} ({})", conference.name, conference.id);
    Ok((StatusCode::CREATED, Json(ConferenceResponse::from(conference))))
}

/// Update a conference (organizer only)
pub async fn update_conference(
    State(state): State<AppState>,
    Path(conference_id): Path<Uuid>,
    Json(payload): Json<UpdateConferenceRequest>,
) -> Result<Json<ConferenceResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the existing conference
    let existing_conference = sqlx::query_as::<_, Conference>(
        r#"
        SELECT * FROM conferences
        WHERE id = $1
        "#,
    )
    .bind(conference_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching conference: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch conference")),
        )
    })?;

    let existing_conference = existing_conference.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Conference not found")),
        )
    })?;

    // Validate name if provided
    if let Some(ref name) = payload.name {
        if name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Conference name cannot be empty")),
            ));
        }
    }

    // Validate dates if provided
    let start_date = payload.start_date.unwrap_or(existing_conference.start_date);
    let end_date = payload.end_date.unwrap_or(existing_conference.end_date);

    if end_date < start_date {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("End date must be after start date")),
        ));
    }

    // Update the conference
    let name = payload
        .name
        .as_ref()
        .map(|s| s.trim())
        .unwrap_or(&existing_conference.name);
    let description = payload
        .description
        .as_ref()
        .map(|s| Some(s.trim()))
        .unwrap_or(existing_conference.description.as_deref());
    let location = payload
        .location
        .as_ref()
        .map(|s| Some(s.trim()))
        .unwrap_or(existing_conference.location.as_deref());
    let is_active = payload.is_active.unwrap_or(existing_conference.is_active);

    let updated_conference = sqlx::query_as::<_, Conference>(
        r#"
        UPDATE conferences
        SET name = $1, description = $2, start_date = $3, end_date = $4,
            location = $5, is_active = $6, updated_at = $7
        WHERE id = $8
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(start_date)
    .bind(end_date)
    .bind(location)
    .bind(is_active)
    .bind(Utc::now())
    .bind(conference_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating conference: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update conference")),
        )
    })?;

    tracing::info!(
        "Conference updated: {} ({})",
        updated_conference.name,
        updated_conference.id
    );
    Ok(Json(ConferenceResponse::from(updated_conference)))
}

/// Delete a conference (organizer only)
pub async fn delete_conference(
    State(state): State<AppState>,
    Path(conference_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query(
        r#"
        DELETE FROM conferences
        WHERE id = $1
        "#,
    )
    .bind(conference_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error deleting conference: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to delete conference")),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Conference not found")),
        ));
    }

    tracing::info!("Conference deleted: {}", conference_id);
    Ok(StatusCode::NO_CONTENT)
}
