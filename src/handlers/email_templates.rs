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
        auth::ErrorResponse, CreateEmailTemplateRequest, EmailTemplate, EmailTemplateResponse,
        UpdateEmailTemplateRequest,
    },
};

/// List all email templates for a conference (organizer-only)
pub async fn list_email_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<EmailTemplateResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let templates = sqlx::query_as::<_, EmailTemplate>(
        r#"
        SELECT * FROM email_templates
        ORDER BY template_type ASC, created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching email templates: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch email templates")),
        )
    })?;

    let responses: Vec<EmailTemplateResponse> = templates
        .into_iter()
        .map(EmailTemplateResponse::from)
        .collect();
    Ok(Json(responses))
}

/// Get a single email template by ID (organizer-only)
pub async fn get_email_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<EmailTemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let template = sqlx::query_as::<_, EmailTemplate>(
        r#"
        SELECT * FROM email_templates
        WHERE id = $1
        "#,
    )
    .bind(template_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching email template: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch email template")),
        )
    })?;

    let template = template.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Email template not found")),
        )
    })?;

    Ok(Json(EmailTemplateResponse::from(template)))
}

/// Create a new email template (organizer-only)
pub async fn create_email_template(
    State(state): State<AppState>,
    Json(payload): Json<CreateEmailTemplateRequest>,
) -> Result<(StatusCode, Json<EmailTemplateResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate required fields
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Name is required")),
        ));
    }

    if payload.subject.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Subject is required")),
        ));
    }

    if payload.body.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Body is required")),
        ));
    }

    // Validate template_type is a valid enum value
    let valid_types = vec![
        "submission_confirmation",
        "talk_accepted",
        "talk_rejected",
        "talk_pending",
        "schedule_notification",
        "custom",
    ];
    if !valid_types.contains(&payload.template_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(&format!(
                "Invalid template type. Must be one of: {}",
                valid_types.join(", ")
            ))),
        ));
    }

    // Create the email template
    let template = sqlx::query_as::<_, EmailTemplate>(
        r#"
        INSERT INTO email_templates (conference_id, template_type, name, subject, body, is_default)
        VALUES ($1, $2::email_template_type, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(payload.conference_id)
    .bind(&payload.template_type)
    .bind(payload.name.trim())
    .bind(payload.subject.trim())
    .bind(payload.body.trim())
    .bind(payload.is_default.unwrap_or(false))
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating email template: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create email template")),
        )
    })?;

    tracing::info!("Email template created: {}", template.id);
    Ok((
        StatusCode::CREATED,
        Json(EmailTemplateResponse::from(template)),
    ))
}

/// Update an existing email template (organizer-only)
pub async fn update_email_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<UpdateEmailTemplateRequest>,
) -> Result<Json<EmailTemplateResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the existing template
    let existing_template = sqlx::query_as::<_, EmailTemplate>(
        r#"
        SELECT * FROM email_templates
        WHERE id = $1
        "#,
    )
    .bind(template_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching email template: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch email template")),
        )
    })?;

    let existing_template = existing_template.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Email template not found")),
        )
    })?;

    // Build update with provided fields
    let name = payload
        .name
        .as_ref()
        .map(|s| s.trim())
        .unwrap_or(&existing_template.name);
    let subject = payload
        .subject
        .as_ref()
        .map(|s| s.trim())
        .unwrap_or(&existing_template.subject);
    let body = payload
        .body
        .as_ref()
        .map(|s| s.trim())
        .unwrap_or(&existing_template.body);
    let is_default = payload.is_default.unwrap_or(existing_template.is_default);

    // Update the template
    let updated_template = sqlx::query_as::<_, EmailTemplate>(
        r#"
        UPDATE email_templates
        SET name = $1,
            subject = $2,
            body = $3,
            is_default = $4,
            updated_at = $5
        WHERE id = $6
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(subject)
    .bind(body)
    .bind(is_default)
    .bind(Utc::now())
    .bind(template_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating email template: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update email template")),
        )
    })?;

    tracing::info!("Email template updated: {}", template_id);
    Ok(Json(EmailTemplateResponse::from(updated_template)))
}

/// Delete an email template (organizer-only)
pub async fn delete_email_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM email_templates WHERE id = $1")
        .bind(template_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error deleting email template: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to delete email template")),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Email template not found")),
        ));
    }

    tracing::info!("Email template deleted: {}", template_id);
    Ok(StatusCode::NO_CONTENT)
}
