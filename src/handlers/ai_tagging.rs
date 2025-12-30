use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::AppState,
    models::label::Label,
};

#[derive(Debug, Deserialize)]
pub struct AutoTagQuery {
    /// Optional state filter for talks to analyze
    pub state: Option<String>,
    /// AI provider to use: "claude" or "openai"
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AutoTagResponse {
    pub suggested_labels: Vec<String>,
    pub existing_labels: Vec<String>,
    pub new_labels: Vec<String>,
}

pub async fn auto_tag_with_claude(
    State(state): State<AppState>,
    Query(params): Query<AutoTagQuery>,
) -> Result<Json<AutoTagResponse>, (StatusCode, String)> {
    // Determine which provider to use (default to Claude for backwards compatibility)
    let provider = params.provider.as_deref().unwrap_or("claude");

    // Check if the selected AI provider is configured
    match provider {
        "claude" => {
            if !state.claude_service.is_configured() {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Claude API is not configured. Please set CLAUDE_API_KEY environment variable."
                        .to_string(),
                ));
            }
        }
        "openai" => {
            if !state.openai_service.is_configured() {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "OpenAI API is not configured. Please set OPENAI_API_KEY environment variable."
                        .to_string(),
                ));
            }
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid provider: {}. Must be 'claude' or 'openai'", provider),
            ));
        }
    }

    // Build query to fetch talks (reuse logic from export handler)
    let mut query = String::from(
        r#"
        SELECT
            t.id,
            t.title,
            t.short_summary,
            t.long_description,
            t.state::text as state,
            u.full_name as speaker_name
        FROM talks t
        INNER JOIN users u ON t.speaker_id = u.id
        "#,
    );

    // Add state filter if provided
    if let Some(state_filter) = &params.state {
        let state_str = match state_filter.to_lowercase().as_str() {
            "submitted" => "submitted",
            "pending" => "pending",
            "accepted" => "accepted",
            "rejected" => "rejected",
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Invalid state filter: {}", state_filter),
                ))
            }
        };
        query.push_str(&format!(" WHERE t.state = '{}'::talk_state", state_str));
    }

    query.push_str(" ORDER BY t.submitted_at DESC LIMIT 50");

    let rows = sqlx::query(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    if rows.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No talks found to analyze".to_string(),
        ));
    }

    // Build simplified JSON for Claude
    let mut talks = Vec::new();
    for row in rows {
        let talk = serde_json::json!({
            "title": row.try_get::<String, _>("title").unwrap_or_default(),
            "summary": row.try_get::<String, _>("short_summary").unwrap_or_default(),
            "description": row.try_get::<Option<String>, _>("long_description").ok().flatten(),
        });
        talks.push(talk);
    }

    let talks_json = serde_json::to_string_pretty(&talks)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize talks: {}", e)))?;

    // Call the appropriate AI API
    let suggested_labels = match provider {
        "claude" => {
            state
                .claude_service
                .suggest_labels(&talks_json)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        }
        "openai" => {
            state
                .openai_service
                .suggest_labels(&talks_json)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        }
        _ => unreachable!(), // Already validated above
    };

    // Fetch existing labels from database
    let existing_labels_db = sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE is_ai_generated = false")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let existing_label_names: Vec<String> = existing_labels_db
        .iter()
        .map(|l| l.name.to_lowercase())
        .collect();

    // Determine which labels are new
    let mut new_labels = Vec::new();
    for label in &suggested_labels {
        if !existing_label_names.contains(&label.to_lowercase()) {
            new_labels.push(label.clone());
        }
    }

    Ok(Json(AutoTagResponse {
        suggested_labels: suggested_labels.clone(),
        existing_labels: suggested_labels
            .iter()
            .filter(|l| existing_label_names.contains(&l.to_lowercase()))
            .cloned()
            .collect(),
        new_labels,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CreateLabelsRequest {
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateLabelsResponse {
    pub created: Vec<Label>,
    pub skipped: Vec<String>,
}

pub async fn create_ai_labels(
    State(state): State<AppState>,
    Json(payload): Json<CreateLabelsRequest>,
) -> Result<Json<CreateLabelsResponse>, (StatusCode, String)> {
    let mut created = Vec::new();
    let mut skipped = Vec::new();

    for label_name in payload.labels {
        // Check if label already exists
        let existing = sqlx::query_as::<_, Label>(
            "SELECT * FROM labels WHERE LOWER(name) = LOWER($1)",
        )
        .bind(&label_name)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

        if existing.is_some() {
            skipped.push(label_name);
            continue;
        }

        // Create new label
        let label = sqlx::query_as::<_, Label>(
            r#"
            INSERT INTO labels (name, description, is_ai_generated)
            VALUES ($1, $2, true)
            RETURNING *
            "#,
        )
        .bind(&label_name)
        .bind(format!("AI-generated label for {}", label_name))
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create label: {}", e)))?;

        created.push(label);
    }

    Ok(Json(CreateLabelsResponse { created, skipped }))
}
