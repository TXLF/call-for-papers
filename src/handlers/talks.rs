use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, CreateTalkRequest, Talk, TalkResponse, TalkState, UpdateTalkRequest,
        User,
    },
};

/// Create a new talk submission
pub async fn create_talk(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateTalkRequest>,
) -> Result<(StatusCode, Json<TalkResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate required fields
    if payload.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Title is required")),
        ));
    }

    if payload.short_summary.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Short summary is required")),
        ));
    }

    // Title length validation
    if payload.title.len() > 500 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Title must be 500 characters or less")),
        ));
    }

    // Create the talk
    let talk = sqlx::query_as::<_, Talk>(
        r#"
        INSERT INTO talks (speaker_id, title, short_summary, long_description, state)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(user.id)
    .bind(payload.title.trim())
    .bind(payload.short_summary.trim())
    .bind(payload.long_description.as_ref().map(|s| s.trim()))
    .bind(TalkState::Submitted)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating talk: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create talk")),
        )
    })?;

    Ok((StatusCode::CREATED, Json(TalkResponse::from(talk))))
}

/// Get all talks for the current user
pub async fn get_my_talks(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<TalkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let talks = sqlx::query_as::<_, Talk>(
        r#"
        SELECT * FROM talks
        WHERE speaker_id = $1
        ORDER BY submitted_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching talks: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch talks")),
        )
    })?;

    let responses: Vec<TalkResponse> = talks.into_iter().map(TalkResponse::from).collect();
    Ok(Json(responses))
}

/// Get a single talk by ID
pub async fn get_talk(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
) -> Result<Json<TalkResponse>, (StatusCode, Json<ErrorResponse>)> {
    let talk = sqlx::query_as::<_, Talk>(
        r#"
        SELECT * FROM talks
        WHERE id = $1
        "#,
    )
    .bind(talk_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching talk: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch talk")),
        )
    })?;

    let talk = talk.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        )
    })?;

    // Check if user can view this talk (speaker or organizer)
    if talk.speaker_id != user.id && !user.is_organizer {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new("You don't have permission to view this talk")),
        ));
    }

    Ok(Json(TalkResponse::from(talk)))
}

/// Update a talk (only by the speaker who created it)
pub async fn update_talk(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
    Json(payload): Json<UpdateTalkRequest>,
) -> Result<Json<TalkResponse>, (StatusCode, Json<ErrorResponse>)> {
    // First, fetch the talk to verify ownership
    let existing_talk = sqlx::query_as::<_, Talk>(
        r#"
        SELECT * FROM talks
        WHERE id = $1
        "#,
    )
    .bind(talk_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching talk: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch talk")),
        )
    })?;

    let existing_talk = existing_talk.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        )
    })?;

    // Verify ownership
    if existing_talk.speaker_id != user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "You can only update your own talk submissions",
            )),
        ));
    }

    // Validate title length if provided
    if let Some(ref title) = payload.title {
        if title.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Title cannot be empty")),
            ));
        }
        if title.len() > 500 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Title must be 500 characters or less")),
            ));
        }
    }

    // Validate short summary if provided
    if let Some(ref summary) = payload.short_summary {
        if summary.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Short summary cannot be empty")),
            ));
        }
    }

    // Build the update query dynamically based on what's provided
    let title = payload
        .title
        .as_ref()
        .map(|s| s.trim())
        .unwrap_or(&existing_talk.title);
    let short_summary = payload
        .short_summary
        .as_ref()
        .map(|s| s.trim())
        .unwrap_or(&existing_talk.short_summary);
    let long_description = match payload.long_description {
        Some(desc) => Some(desc.trim().to_string()),
        None => existing_talk.long_description.clone(),
    };
    let slides_url = match payload.slides_url {
        Some(url) => Some(url),
        None => existing_talk.slides_url.clone(),
    };

    let updated_talk = sqlx::query_as::<_, Talk>(
        r#"
        UPDATE talks
        SET title = $1,
            short_summary = $2,
            long_description = $3,
            slides_url = $4,
            updated_at = $5
        WHERE id = $6
        RETURNING *
        "#,
    )
    .bind(title)
    .bind(short_summary)
    .bind(long_description)
    .bind(slides_url)
    .bind(Utc::now())
    .bind(talk_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating talk: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update talk")),
        )
    })?;

    Ok(Json(TalkResponse::from(updated_talk)))
}

/// Delete a talk (only by the speaker who created it)
pub async fn delete_talk(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // First, verify the talk exists and user owns it
    let talk = sqlx::query_as::<_, Talk>(
        r#"
        SELECT * FROM talks
        WHERE id = $1
        "#,
    )
    .bind(talk_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching talk: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch talk")),
        )
    })?;

    let talk = talk.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        )
    })?;

    // Verify ownership
    if talk.speaker_id != user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "You can only delete your own talk submissions",
            )),
        ));
    }

    // Delete the talk
    sqlx::query("DELETE FROM talks WHERE id = $1")
        .bind(talk_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error deleting talk: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to delete talk")),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}
