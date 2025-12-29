use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, CreateTrackRequest, Track, TrackResponse, UpdateTrackRequest,
    },
};

/// List all tracks (public endpoint)
pub async fn list_tracks(
    State(state): State<AppState>,
) -> Result<Json<Vec<TrackResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let tracks = sqlx::query_as::<_, Track>(
        r#"
        SELECT * FROM tracks
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching tracks: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch tracks")),
        )
    })?;

    let responses: Vec<TrackResponse> = tracks.into_iter().map(TrackResponse::from).collect();
    Ok(Json(responses))
}

/// Get a single track by ID (public endpoint)
pub async fn get_track(
    State(state): State<AppState>,
    Path(track_id): Path<Uuid>,
) -> Result<Json<TrackResponse>, (StatusCode, Json<ErrorResponse>)> {
    let track = sqlx::query_as::<_, Track>(
        r#"
        SELECT * FROM tracks
        WHERE id = $1
        "#,
    )
    .bind(track_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching track: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch track")),
        )
    })?;

    let track = track.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Track not found")),
        )
    })?;

    Ok(Json(TrackResponse::from(track)))
}

/// Create a new track (organizer only)
pub async fn create_track(
    State(state): State<AppState>,
    Json(payload): Json<CreateTrackRequest>,
) -> Result<(StatusCode, Json<TrackResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate name
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Track name is required")),
        ));
    }

    // Validate capacity if provided
    if let Some(capacity) = payload.capacity {
        if capacity < 1 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Capacity must be at least 1")),
            ));
        }
    }

    // Create the track
    let track = sqlx::query_as::<_, Track>(
        r#"
        INSERT INTO tracks (conference_id, name, description, capacity)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(payload.conference_id)
    .bind(payload.name.trim())
    .bind(payload.description.as_ref().map(|s| s.trim()))
    .bind(payload.capacity)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating track: {}", e);
        // Check for foreign key violation
        if let Some(db_err) = e.as_database_error() {
            if db_err.message().contains("foreign key") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("Invalid conference ID")),
                );
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create track")),
        )
    })?;

    tracing::info!("Track created: {} ({})", track.name, track.id);
    Ok((StatusCode::CREATED, Json(TrackResponse::from(track))))
}

/// Update a track (organizer only)
pub async fn update_track(
    State(state): State<AppState>,
    Path(track_id): Path<Uuid>,
    Json(payload): Json<UpdateTrackRequest>,
) -> Result<Json<TrackResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the existing track
    let existing_track = sqlx::query_as::<_, Track>(
        r#"
        SELECT * FROM tracks
        WHERE id = $1
        "#,
    )
    .bind(track_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching track: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch track")),
        )
    })?;

    let existing_track = existing_track.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Track not found")),
        )
    })?;

    // Validate name if provided
    if let Some(ref name) = payload.name {
        if name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Track name cannot be empty")),
            ));
        }
    }

    // Validate capacity if provided
    if let Some(capacity) = payload.capacity {
        if capacity < 1 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Capacity must be at least 1")),
            ));
        }
    }

    // Update the track
    let name = payload.name.as_ref().map(|s| s.trim()).unwrap_or(&existing_track.name);
    let description = payload
        .description
        .as_ref()
        .map(|s| Some(s.trim()))
        .unwrap_or(existing_track.description.as_deref());
    let capacity = payload.capacity.or(existing_track.capacity);

    let updated_track = sqlx::query_as::<_, Track>(
        r#"
        UPDATE tracks
        SET name = $1, description = $2, capacity = $3
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(capacity)
    .bind(track_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating track: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update track")),
        )
    })?;

    tracing::info!("Track updated: {} ({})", updated_track.name, updated_track.id);
    Ok(Json(TrackResponse::from(updated_track)))
}

/// Delete a track (organizer only)
pub async fn delete_track(
    State(state): State<AppState>,
    Path(track_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query(
        r#"
        DELETE FROM tracks
        WHERE id = $1
        "#,
    )
    .bind(track_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error deleting track: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to delete track")),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Track not found")),
        ));
    }

    tracing::info!("Track deleted: {}", track_id);
    Ok(StatusCode::NO_CONTENT)
}
