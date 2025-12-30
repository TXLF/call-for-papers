use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, CreateTalkRequest, Label, LabelResponse, Talk, TalkResponse,
        TalkState, UpdateTalkRequest, RespondToTalkRequest, TalkAction, ChangeStateRequest, User,
    },
};

#[derive(Debug, Deserialize)]
pub struct ListTalksQuery {
    pub state: Option<String>,
}

/// Helper function to fetch labels for a talk
async fn fetch_talk_labels(
    db: &sqlx::PgPool,
    talk_id: Uuid,
) -> Result<Vec<LabelResponse>, sqlx::Error> {
    let labels = sqlx::query_as::<_, Label>(
        r#"
        SELECT l.* FROM labels l
        INNER JOIN talk_labels tl ON l.id = tl.label_id
        WHERE tl.talk_id = $1
        ORDER BY l.name ASC
        "#,
    )
    .bind(talk_id)
    .fetch_all(db)
    .await?;

    Ok(labels.into_iter().map(LabelResponse::from).collect())
}

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

    // Add labels if provided
    if let Some(label_ids) = payload.label_ids {
        for label_id in label_ids {
            let _ = sqlx::query(
                r#"
                INSERT INTO talk_labels (talk_id, label_id, added_by)
                VALUES ($1, $2, $3)
                ON CONFLICT (talk_id, label_id) DO NOTHING
                "#,
            )
            .bind(talk.id)
            .bind(label_id)
            .bind(user.id)
            .execute(&state.db)
            .await;
        }
    }

    // Fetch labels and return
    let labels = fetch_talk_labels(&state.db, talk.id).await.unwrap_or_default();
    let response = TalkResponse::from(talk).with_labels(labels);

    Ok((StatusCode::CREATED, Json(response)))
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

    // Fetch labels for each talk
    let mut responses = Vec::new();
    for talk in talks {
        let labels = fetch_talk_labels(&state.db, talk.id).await.unwrap_or_default();
        responses.push(TalkResponse::from(talk).with_labels(labels));
    }

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

    // Fetch labels for the talk
    let labels = fetch_talk_labels(&state.db, talk.id).await.unwrap_or_default();
    let response = TalkResponse::from(talk).with_labels(labels);

    Ok(Json(response))
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

/// Upload slides for a talk
pub async fn upload_slides(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<TalkResponse>, (StatusCode, Json<ErrorResponse>)> {
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
                "You can only upload slides for your own talk submissions",
            )),
        ));
    }

    // Process the multipart form data
    let mut file_path: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            tracing::error!("Error reading multipart field: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Invalid file upload")),
            )
        })?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "slides" {
            continue;
        }

        let file_name = field
            .file_name()
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("No filename provided")),
                )
            })?
            .to_string();

        // Validate file extension
        let allowed_extensions = ["pdf", "ppt", "pptx", "key", "odp"];
        let path_buf = PathBuf::from(&file_name);
        let extension = path_buf
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !allowed_extensions.contains(&extension.to_lowercase().as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "Invalid file type. Allowed: pdf, ppt, pptx, key, odp",
                )),
            ));
        }

        // Generate unique filename
        let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);
        let file_path_buf = PathBuf::from(&state.config.upload_dir).join(&unique_name);

        // Read file data
        let data = field.bytes().await.map_err(|e| {
            tracing::error!("Error reading file data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to read file")),
            )
        })?;

        // Validate file size (max 50MB)
        const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
        if data.len() > MAX_FILE_SIZE {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("File size exceeds 50MB limit")),
            ));
        }

        // Save file to disk
        tokio::fs::write(&file_path_buf, data)
            .await
            .map_err(|e| {
                tracing::error!("Error saving file: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to save file")),
                )
            })?;

        file_path = Some(format!("/uploads/{}", unique_name));
        break;
    }

    let slides_url = file_path.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("No file uploaded")),
        )
    })?;

    // Update the talk with the slides URL
    let updated_talk = sqlx::query_as::<_, Talk>(
        r#"
        UPDATE talks
        SET slides_url = $1,
            updated_at = $2
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(&slides_url)
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

/// Respond to a pending talk (accept or decline)
pub async fn respond_to_talk(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
    Json(payload): Json<RespondToTalkRequest>,
) -> Result<Json<TalkResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the talk to verify existence and ownership
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

    // Verify ownership - only the speaker can respond
    if talk.speaker_id != user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "You can only respond to your own talk submissions",
            )),
        ));
    }

    // Verify the talk is in pending state
    if !matches!(talk.state, TalkState::Pending) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "You can only respond to talks in pending state",
            )),
        ));
    }

    // Determine the new state based on action
    let new_state = match payload.action {
        TalkAction::Accept => TalkState::Accepted,
        TalkAction::Decline => TalkState::Rejected,
    };

    // Update the talk state
    let updated_talk = sqlx::query_as::<_, Talk>(
        r#"
        UPDATE talks
        SET state = $1,
            updated_at = $2
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(new_state)
    .bind(Utc::now())
    .bind(talk_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating talk state: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update talk state")),
        )
    })?;

    Ok(Json(TalkResponse::from(updated_talk)))
}

/// Change talk state (organizer-only)
pub async fn change_talk_state(
    State(state): State<AppState>,
    Path(talk_id): Path<Uuid>,
    Json(payload): Json<ChangeStateRequest>,
) -> Result<Json<TalkResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the talk to verify existence and current state
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

    // Validate state transition
    if !talk.state.can_transition_to(&payload.new_state) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(&format!(
                "Invalid state transition: cannot move from {:?} to {:?}",
                talk.state, payload.new_state
            ))),
        ));
    }

    // Update the talk state
    let updated_talk = sqlx::query_as::<_, Talk>(
        r#"
        UPDATE talks
        SET state = $1,
            updated_at = $2
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(&payload.new_state)
    .bind(Utc::now())
    .bind(talk_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating talk state: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update talk state")),
        )
    })?;

    tracing::info!(
        "Talk {} state changed from {:?} to {:?}{}",
        talk_id,
        talk.state,
        payload.new_state,
        payload.reason.as_ref().map(|r| format!(" (reason: {})", r)).unwrap_or_default()
    );

    // Send email notification to speaker about state change
    if state.email_service.is_configured() {
        // Fetch speaker info
        let speaker_result: Result<(String, String), sqlx::Error> = sqlx::query_as(
            r#"
            SELECT full_name, email FROM users
            WHERE id = $1
            "#,
        )
        .bind(updated_talk.speaker_id)
        .fetch_one(&state.db)
        .await;

        if let Ok((speaker_name, speaker_email)) = speaker_result {
            // Determine template type based on new state
            let template_type = match payload.new_state {
                TalkState::Pending => "talk_pending",
                TalkState::Accepted => "talk_accepted",
                TalkState::Rejected => "talk_rejected",
                _ => "", // No email for other states
            };

            if !template_type.is_empty() {
                // Get conference_id from talk (we'll need to add this to Talk struct or fetch separately)
                // For now, use a default/active conference
                if let Ok(conf) = crate::handlers::conferences::get_active_conference_internal(&state.db).await {
                    let variables = crate::services::email::EmailVariables {
                        speaker_name,
                        speaker_email: speaker_email.clone(),
                        talk_title: updated_talk.title.clone(),
                        talk_id: updated_talk.id.to_string(),
                        reason: payload.reason.clone(),
                        schedule_date: None,
                        schedule_time: None,
                        track_name: None,
                    };

                    // Send email asynchronously (don't block on errors)
                    let email_result = state
                        .email_service
                        .send_templated_email(
                            conf.id,
                            template_type,
                            &speaker_email,
                            variables,
                            Some(updated_talk.id),
                            None, // No specific sender (system-generated)
                        )
                        .await;

                    if let Err(e) = email_result {
                        tracing::warn!("Failed to send email notification: {}", e);
                    }
                }
            }
        }
    }

    Ok(Json(TalkResponse::from(updated_talk)))
}

/// List all talks (organizer-only) with optional state filtering
pub async fn list_all_talks(
    State(state): State<AppState>,
    Query(query): Query<ListTalksQuery>,
) -> Result<Json<Vec<TalkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Build query with optional state filtering
    let talks = if let Some(state_filter) = query.state {
        // Parse the state filter
        let talk_state = match state_filter.to_lowercase().as_str() {
            "submitted" => TalkState::Submitted,
            "pending" => TalkState::Pending,
            "accepted" => TalkState::Accepted,
            "rejected" => TalkState::Rejected,
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("Invalid state filter. Use: submitted, pending, accepted, or rejected")),
                ));
            }
        };

        sqlx::query_as::<_, Talk>(
            r#"
            SELECT t.* FROM talks t
            WHERE t.state = $1
            ORDER BY t.submitted_at DESC
            "#,
        )
        .bind(talk_state)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, Talk>(
            r#"
            SELECT t.* FROM talks t
            ORDER BY t.submitted_at DESC
            "#,
        )
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| {
        tracing::error!("Database error fetching talks: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch talks")),
        )
    })?;

    // Fetch speaker info and labels for each talk
    let mut responses = Vec::new();
    for talk in talks {
        // Fetch speaker info
        let speaker: (String, String) = sqlx::query_as(
            r#"
            SELECT full_name, email FROM users
            WHERE id = $1
            "#,
        )
        .bind(talk.speaker_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching speaker: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to fetch speaker information")),
            )
        })?;

        // Fetch labels
        let labels = fetch_talk_labels(&state.db, talk.id).await.unwrap_or_default();

        // Build response with speaker info and labels
        let response = TalkResponse::from(talk)
            .with_speaker_info(speaker.0, speaker.1)
            .with_labels(labels);

        responses.push(response);
    }

    Ok(Json(responses))
}
