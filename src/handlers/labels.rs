use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, AddLabelToTalkRequest, CreateLabelRequest, Label, LabelResponse,
        UpdateLabelRequest, User,
    },
};

/// List all labels (public endpoint)
pub async fn list_labels(
    State(state): State<AppState>,
) -> Result<Json<Vec<LabelResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let labels = sqlx::query_as::<_, Label>(
        r#"
        SELECT * FROM labels
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching labels: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch labels")),
        )
    })?;

    let responses: Vec<LabelResponse> = labels.into_iter().map(LabelResponse::from).collect();
    Ok(Json(responses))
}

/// Create a new label (organizer only)
pub async fn create_label(
    State(state): State<AppState>,
    Json(payload): Json<CreateLabelRequest>,
) -> Result<(StatusCode, Json<LabelResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate name
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Label name is required")),
        ));
    }

    // Validate color format if provided (hex color)
    if let Some(ref color) = payload.color {
        if !color.starts_with('#') || (color.len() != 4 && color.len() != 7) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "Color must be a valid hex color (e.g., #FF5733 or #F57)",
                )),
            ));
        }
    }

    // Create the label
    let label = sqlx::query_as::<_, Label>(
        r#"
        INSERT INTO labels (name, description, color, is_ai_generated)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.description.as_ref().map(|s| s.trim()))
    .bind(payload.color)
    .bind(false) // User-created labels are not AI-generated
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating label: {}", e);
        // Check for unique constraint violation
        if let Some(db_err) = e.as_database_error() {
            if db_err.is_unique_violation() {
                return (
                    StatusCode::CONFLICT,
                    Json(ErrorResponse::new("A label with this name already exists")),
                );
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create label")),
        )
    })?;

    Ok((StatusCode::CREATED, Json(LabelResponse::from(label))))
}

/// Update a label (organizer only)
pub async fn update_label(
    State(state): State<AppState>,
    Path(label_id): Path<Uuid>,
    Json(payload): Json<UpdateLabelRequest>,
) -> Result<Json<LabelResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch existing label
    let existing_label = sqlx::query_as::<_, Label>(
        r#"
        SELECT * FROM labels WHERE id = $1
        "#,
    )
    .bind(label_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching label: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch label")),
        )
    })?;

    let existing_label = existing_label.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Label not found")),
        )
    })?;

    // Validate name if provided
    if let Some(ref name) = payload.name {
        if name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Label name cannot be empty")),
            ));
        }
    }

    // Validate color if provided
    if let Some(ref color) = payload.color {
        if !color.starts_with('#') || (color.len() != 4 && color.len() != 7) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "Color must be a valid hex color (e.g., #FF5733 or #F57)",
                )),
            ));
        }
    }

    // Update the label
    let name = payload
        .name
        .as_ref()
        .map(|s| s.trim())
        .unwrap_or(&existing_label.name);
    let description = match payload.description {
        Some(desc) => Some(desc.trim().to_string()),
        None => existing_label.description.clone(),
    };
    let color = match payload.color {
        Some(c) => Some(c),
        None => existing_label.color.clone(),
    };

    let updated_label = sqlx::query_as::<_, Label>(
        r#"
        UPDATE labels
        SET name = $1, description = $2, color = $3
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(color)
    .bind(label_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating label: {}", e);
        if let Some(db_err) = e.as_database_error() {
            if db_err.is_unique_violation() {
                return (
                    StatusCode::CONFLICT,
                    Json(ErrorResponse::new("A label with this name already exists")),
                );
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update label")),
        )
    })?;

    Ok(Json(LabelResponse::from(updated_label)))
}

/// Delete a label (organizer only)
pub async fn delete_label(
    State(state): State<AppState>,
    Path(label_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Check if label exists
    let label_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(SELECT 1 FROM labels WHERE id = $1)
        "#,
    )
    .bind(label_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking label: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to check label")),
        )
    })?;

    if !label_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Label not found")),
        ));
    }

    // Delete the label (cascade will handle talk_labels)
    sqlx::query("DELETE FROM labels WHERE id = $1")
        .bind(label_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error deleting label: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to delete label")),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get labels for a specific talk
pub async fn get_talk_labels(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
) -> Result<Json<Vec<LabelResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // First check if user has permission to view this talk
    let talk = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT speaker_id FROM talks WHERE id = $1
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

    let speaker_id = talk.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        )
    })?;

    // Check permission
    if speaker_id != user.id && !user.is_organizer {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "You don't have permission to view this talk's labels",
            )),
        ));
    }

    // Fetch labels for the talk
    let labels = sqlx::query_as::<_, Label>(
        r#"
        SELECT l.* FROM labels l
        INNER JOIN talk_labels tl ON l.id = tl.label_id
        WHERE tl.talk_id = $1
        ORDER BY l.name ASC
        "#,
    )
    .bind(talk_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching talk labels: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch talk labels")),
        )
    })?;

    let responses: Vec<LabelResponse> = labels.into_iter().map(LabelResponse::from).collect();
    Ok(Json(responses))
}

/// Add labels to a talk
pub async fn add_labels_to_talk(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
    Json(payload): Json<AddLabelToTalkRequest>,
) -> Result<Json<Vec<LabelResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // First check if user has permission to modify this talk
    let talk = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT speaker_id FROM talks WHERE id = $1
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

    let speaker_id = talk.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        )
    })?;

    // Check permission
    if speaker_id != user.id && !user.is_organizer {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "You can only add labels to your own talks",
            )),
        ));
    }

    if payload.label_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("At least one label ID must be provided")),
        ));
    }

    // Insert labels (ON CONFLICT DO NOTHING to handle duplicates gracefully)
    for label_id in &payload.label_ids {
        sqlx::query(
            r#"
            INSERT INTO talk_labels (talk_id, label_id, added_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (talk_id, label_id) DO NOTHING
            "#,
        )
        .bind(talk_id)
        .bind(label_id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error adding label to talk: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to add labels to talk")),
            )
        })?;
    }

    // Fetch and return all labels for the talk
    let labels = sqlx::query_as::<_, Label>(
        r#"
        SELECT l.* FROM labels l
        INNER JOIN talk_labels tl ON l.id = tl.label_id
        WHERE tl.talk_id = $1
        ORDER BY l.name ASC
        "#,
    )
    .bind(talk_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching talk labels: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch talk labels")),
        )
    })?;

    let responses: Vec<LabelResponse> = labels.into_iter().map(LabelResponse::from).collect();
    Ok(Json(responses))
}

/// Remove a label from a talk
pub async fn remove_label_from_talk(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((talk_id, label_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // First check if user has permission to modify this talk
    let talk = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT speaker_id FROM talks WHERE id = $1
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

    let speaker_id = talk.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        )
    })?;

    // Check permission
    if speaker_id != user.id && !user.is_organizer {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "You can only remove labels from your own talks",
            )),
        ));
    }

    // Remove the label
    let result = sqlx::query(
        r#"
        DELETE FROM talk_labels
        WHERE talk_id = $1 AND label_id = $2
        "#,
    )
    .bind(talk_id)
    .bind(label_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error removing label from talk: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to remove label from talk")),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Label not found on this talk")),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
