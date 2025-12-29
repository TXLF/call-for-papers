use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{ErrorResponse, TalkState},
};

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_talks: i64,
    pub talks_by_state: TalksByState,
    pub rating_stats: RatingStats,
    pub recent_submissions: Vec<RecentTalk>,
    pub unrated_talks: i64,
}

#[derive(Debug, Serialize)]
pub struct TalksByState {
    pub submitted: i64,
    pub pending: i64,
    pub accepted: i64,
    pub rejected: i64,
}

#[derive(Debug, Serialize)]
pub struct RatingStats {
    pub total_ratings: i64,
    pub average_rating: Option<f64>,
    pub talks_with_ratings: i64,
    pub talks_without_ratings: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RecentTalk {
    pub id: Uuid,
    pub title: String,
    pub speaker_name: String,
    pub state: TalkState,
    pub submitted_at: DateTime<Utc>,
    pub rating_count: Option<i64>,
    pub average_rating: Option<f64>,
}

/// Get dashboard statistics (organizer only)
pub async fn get_dashboard_stats(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, (StatusCode, Json<ErrorResponse>)> {
    // Get total talks count
    let total_talks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM talks")
        .fetch_one(&state.db)
        .await?;

    // Get talks by state
    let state_counts: Vec<(TalkState, i64)> = sqlx::query_as(
        r#"
        SELECT state, COUNT(*) as count
        FROM talks
        GROUP BY state
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut talks_by_state = TalksByState {
        submitted: 0,
        pending: 0,
        accepted: 0,
        rejected: 0,
    };

    for (talk_state, count) in state_counts {
        match talk_state {
            TalkState::Submitted => talks_by_state.submitted = count,
            TalkState::Pending => talks_by_state.pending = count,
            TalkState::Accepted => talks_by_state.accepted = count,
            TalkState::Rejected => talks_by_state.rejected = count,
        }
    }

    // Get rating statistics
    let rating_stats_row: (i64, Option<f64>, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(r.id) as total_ratings,
            AVG(r.rating) as average_rating,
            COUNT(DISTINCT r.talk_id) as talks_with_ratings,
            (SELECT COUNT(*) FROM talks) - COUNT(DISTINCT r.talk_id) as talks_without_ratings
        FROM ratings r
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    let rating_stats = RatingStats {
        total_ratings: rating_stats_row.0,
        average_rating: rating_stats_row.1,
        talks_with_ratings: rating_stats_row.2,
        talks_without_ratings: rating_stats_row.3,
    };

    // Get recent submissions (last 10 talks)
    let recent_submissions: Vec<RecentTalk> = sqlx::query_as(
        r#"
        SELECT
            t.id,
            t.title,
            u.full_name as speaker_name,
            t.state,
            t.submitted_at,
            COUNT(r.id) as rating_count,
            AVG(r.rating) as average_rating
        FROM talks t
        JOIN users u ON t.speaker_id = u.id
        LEFT JOIN ratings r ON t.id = r.talk_id
        GROUP BY t.id, t.title, u.full_name, t.state, t.submitted_at
        ORDER BY t.submitted_at DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    // Unrated talks count
    let unrated_talks = rating_stats.talks_without_ratings;

    Ok(Json(DashboardStats {
        total_talks: total_talks.0,
        talks_by_state,
        rating_stats,
        recent_submissions,
        unrated_talks,
    }))
}
