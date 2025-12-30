use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::AppState,
    models::talk::TalkState,
};

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    /// Optional state filter
    pub state: Option<String>,
    /// Export format (json or csv)
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportedTalk {
    pub id: String,
    pub title: String,
    pub short_summary: String,
    pub long_description: Option<String>,
    pub speaker_name: String,
    pub speaker_email: String,
    pub state: String,
    pub submitted_at: String,
    pub labels: Vec<String>,
    pub average_rating: Option<f64>,
    pub rating_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub talks: Vec<ExportedTalk>,
    pub total_count: usize,
    pub exported_at: String,
}

pub async fn export_talks(
    State(state): State<AppState>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<ExportResponse>, (StatusCode, String)> {
    // Build query based on filters
    let mut query = String::from(
        r#"
        SELECT
            t.id,
            t.title,
            t.short_summary,
            t.long_description,
            t.state::text as state,
            t.submitted_at,
            u.full_name as speaker_name,
            u.email as speaker_email,
            COALESCE(
                (SELECT json_agg(l.name)
                 FROM talk_labels tl
                 INNER JOIN labels l ON tl.label_id = l.id
                 WHERE tl.talk_id = t.id),
                '[]'::json
            ) as labels,
            (SELECT AVG(rating)::float FROM ratings WHERE talk_id = t.id) as average_rating,
            (SELECT COUNT(*)::bigint FROM ratings WHERE talk_id = t.id) as rating_count
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

    query.push_str(" ORDER BY t.submitted_at DESC");

    let rows = sqlx::query(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let mut talks = Vec::new();
    for row in rows {
        let labels_json: serde_json::Value = row
            .try_get("labels")
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get labels: {}", e)))?;

        let labels: Vec<String> = serde_json::from_value(labels_json)
            .unwrap_or_default();

        talks.push(ExportedTalk {
            id: row
                .try_get::<Uuid, _>("id")
                .map(|id| id.to_string())
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get id: {}", e)))?,
            title: row
                .try_get("title")
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get title: {}", e)))?,
            short_summary: row
                .try_get("short_summary")
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get short_summary: {}", e)))?,
            long_description: row.try_get("long_description").ok(),
            speaker_name: row
                .try_get("speaker_name")
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get speaker_name: {}", e)))?,
            speaker_email: row
                .try_get("speaker_email")
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get speaker_email: {}", e)))?,
            state: row
                .try_get("state")
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get state: {}", e)))?,
            submitted_at: row
                .try_get::<chrono::NaiveDateTime, _>("submitted_at")
                .map(|dt| dt.to_string())
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get submitted_at: {}", e)))?,
            labels,
            average_rating: row.try_get("average_rating").ok(),
            rating_count: row.try_get("rating_count").unwrap_or(0),
        });
    }

    let total_count = talks.len();
    let exported_at = chrono::Utc::now().to_rfc3339();

    Ok(Json(ExportResponse {
        talks,
        total_count,
        exported_at,
    }))
}
