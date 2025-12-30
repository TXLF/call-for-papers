use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::talk::TalkState,
    services::email::EmailVariables,
};

#[derive(Debug, Deserialize)]
pub struct BulkEmailRequest {
    /// Filter recipients by talk state
    pub filter_by_state: Option<Vec<TalkState>>,
    /// Specific talk IDs to send to (overrides state filter)
    pub talk_ids: Option<Vec<Uuid>>,
    /// Use an email template
    pub template_id: Option<Uuid>,
    /// Or use custom subject and body
    pub custom_subject: Option<String>,
    pub custom_body: Option<String>,
    /// Optional custom variables (will merge with auto-generated ones)
    pub additional_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkEmailResponse {
    pub emails_sent: usize,
    pub failed_emails: usize,
    pub errors: Vec<String>,
}

#[derive(Debug)]
struct EmailRecipient {
    speaker_name: String,
    speaker_email: String,
    talk_id: Uuid,
    talk_title: String,
}

pub async fn send_bulk_email(
    State(state): State<AppState>,
    Json(payload): Json<BulkEmailRequest>,
) -> Result<Json<BulkEmailResponse>, (StatusCode, String)> {
    // Check if email is configured
    if !state.email_service.is_configured() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Email service is not configured".to_string(),
        ));
    }

    // Validate request
    if payload.template_id.is_none()
        && (payload.custom_subject.is_none() || payload.custom_body.is_none())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Either template_id or both custom_subject and custom_body must be provided"
                .to_string(),
        ));
    }

    // Fetch recipients based on filters
    let recipients = fetch_recipients(&state, &payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if recipients.is_empty() {
        return Ok(Json(BulkEmailResponse {
            emails_sent: 0,
            failed_emails: 0,
            errors: vec!["No recipients matched the filter criteria".to_string()],
        }));
    }

    // Get active conference
    let conference = crate::handlers::conferences::get_active_conference_internal(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut emails_sent = 0;
    let mut failed_emails = 0;
    let mut errors = Vec::new();

    // Send email to each recipient
    for recipient in recipients {
        let subject: String;
        let body: String;

        if let Some(template_id) = payload.template_id {
            // Use template
            let template = sqlx::query_as::<_, crate::services::email::EmailTemplate>(
                "SELECT * FROM email_templates WHERE id = $1",
            )
            .bind(template_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::NOT_FOUND,
                    format!("Template not found: {}", e),
                )
            })?;

            subject = template.subject.clone();
            body = template.body.clone();
        } else {
            // Use custom subject and body
            subject = payload.custom_subject.clone().unwrap();
            body = payload.custom_body.clone().unwrap();
        }

        // Prepare variables for template rendering
        let mut variables = EmailVariables {
            speaker_name: recipient.speaker_name.clone(),
            speaker_email: recipient.speaker_email.clone(),
            talk_title: recipient.talk_title.clone(),
            talk_id: recipient.talk_id.to_string(),
            reason: payload.additional_message.clone(),
            schedule_date: None,
            schedule_time: None,
            track_name: None,
        };

        // Render subject and body with variables
        let rendered_subject = match state
            .email_service
            .render_template(&subject, &variables)
        {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!(
                    "Failed to render subject for {}: {}",
                    recipient.speaker_email, e
                ));
                failed_emails += 1;
                continue;
            }
        };

        let rendered_body = match state.email_service.render_template(&body, &variables) {
            Ok(b) => b,
            Err(e) => {
                errors.push(format!(
                    "Failed to render body for {}: {}",
                    recipient.speaker_email, e
                ));
                failed_emails += 1;
                continue;
            }
        };

        // Send email
        match state
            .email_service
            .send_email(
                &recipient.speaker_email,
                &rendered_subject,
                &rendered_body,
                Some(conference.id),
                Some(recipient.talk_id),
                None,
            )
            .await
        {
            Ok(_) => emails_sent += 1,
            Err(e) => {
                errors.push(format!(
                    "Failed to send to {}: {}",
                    recipient.speaker_email, e
                ));
                failed_emails += 1;
            }
        }
    }

    Ok(Json(BulkEmailResponse {
        emails_sent,
        failed_emails,
        errors,
    }))
}

async fn fetch_recipients(
    state: &AppState,
    payload: &BulkEmailRequest,
) -> Result<Vec<EmailRecipient>, String> {
    let mut query = String::from(
        r#"
        SELECT
            u.full_name as speaker_name,
            u.email as speaker_email,
            t.id as talk_id,
            t.title as talk_title
        FROM talks t
        INNER JOIN users u ON t.speaker_id = u.id
        WHERE 1=1
        "#,
    );

    let mut conditions = Vec::new();

    // Filter by specific talk IDs if provided
    if let Some(talk_ids) = &payload.talk_ids {
        if !talk_ids.is_empty() {
            let ids: Vec<String> = talk_ids.iter().map(|id| format!("'{}'", id)).collect();
            conditions.push(format!("t.id IN ({})", ids.join(", ")));
        }
    } else if let Some(states) = &payload.filter_by_state {
        // Filter by talk states
        if !states.is_empty() {
            let state_strings: Vec<String> = states
                .iter()
                .map(|s| {
                    format!(
                        "'{}'",
                        match s {
                            TalkState::Submitted => "submitted",
                            TalkState::Pending => "pending",
                            TalkState::Accepted => "accepted",
                            TalkState::Rejected => "rejected",
                        }
                    )
                })
                .collect();
            conditions.push(format!("t.state IN ({})", state_strings.join(", ")));
        }
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    let rows = sqlx::query(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let mut recipients = Vec::new();
    for row in rows {
        recipients.push(EmailRecipient {
            speaker_name: row
                .try_get("speaker_name")
                .map_err(|e| format!("Failed to get speaker_name: {}", e))?,
            speaker_email: row
                .try_get("speaker_email")
                .map_err(|e| format!("Failed to get speaker_email: {}", e))?,
            talk_id: row
                .try_get("talk_id")
                .map_err(|e| format!("Failed to get talk_id: {}", e))?,
            talk_title: row
                .try_get("talk_title")
                .map_err(|e| format!("Failed to get talk_title: {}", e))?,
        });
    }

    Ok(recipients)
}
