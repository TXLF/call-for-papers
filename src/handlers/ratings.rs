use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, CreateRatingRequest, Rating, RatingDistribution, RatingResponse,
        RatingsStatisticsResponse, TalkRatingStats, User,
    },
};

/// Create or update a rating for a talk (organizer only)
/// This is an upsert operation - if the organizer has already rated this talk, it updates the existing rating
pub async fn create_or_update_rating(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
    Json(payload): Json<CreateRatingRequest>,
) -> Result<Json<RatingResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate rating is within range (1-5)
    if payload.rating < 1 || payload.rating > 5 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Rating must be between 1 and 5")),
        ));
    }

    // Check if the talk exists
    let talk_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(SELECT 1 FROM talks WHERE id = $1)
        "#,
    )
    .bind(talk_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking talk existence: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to verify talk")),
        )
    })?;

    if !talk_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        ));
    }

    // Upsert the rating (insert or update if already exists)
    let rating = sqlx::query_as::<_, Rating>(
        r#"
        INSERT INTO ratings (talk_id, organizer_id, rating, notes)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (talk_id, organizer_id)
        DO UPDATE SET
            rating = EXCLUDED.rating,
            notes = EXCLUDED.notes,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(talk_id)
    .bind(user.id)
    .bind(payload.rating)
    .bind(payload.notes.as_ref().map(|s| s.trim()))
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating/updating rating: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to save rating")),
        )
    })?;

    let response = RatingResponse::from(rating)
        .with_organizer_info(user.full_name.clone(), user.email.clone());

    Ok(Json(response))
}

/// Get all ratings for a talk (organizer only)
pub async fn get_talk_ratings(
    State(state): State<AppState>,
    Path(talk_id): Path<Uuid>,
) -> Result<Json<Vec<RatingResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Check if the talk exists
    let talk_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(SELECT 1 FROM talks WHERE id = $1)
        "#,
    )
    .bind(talk_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking talk existence: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to verify talk")),
        )
    })?;

    if !talk_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Talk not found")),
        ));
    }

    // Fetch all ratings for the talk with organizer information in a single query (optimized)
    #[derive(sqlx::FromRow)]
    struct RatingWithOrganizer {
        id: Uuid,
        talk_id: Uuid,
        organizer_id: Uuid,
        rating: i32,
        notes: Option<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        organizer_name: String,
        organizer_email: String,
    }

    let ratings_with_organizers = sqlx::query_as::<_, RatingWithOrganizer>(
        r#"
        SELECT
            r.id,
            r.talk_id,
            r.organizer_id,
            r.rating,
            r.notes,
            r.created_at,
            r.updated_at,
            u.full_name as organizer_name,
            u.email as organizer_email
        FROM ratings r
        JOIN users u ON r.organizer_id = u.id
        WHERE r.talk_id = $1
        ORDER BY r.created_at DESC
        "#,
    )
    .bind(talk_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching ratings: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch ratings")),
        )
    })?;

    // Convert to response format
    let responses: Vec<RatingResponse> = ratings_with_organizers
        .into_iter()
        .map(|r| RatingResponse {
            id: r.id,
            talk_id: r.talk_id,
            organizer_id: r.organizer_id,
            organizer_name: r.organizer_name,
            organizer_email: r.organizer_email,
            rating: r.rating,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect();

    Ok(Json(responses))
}

/// Delete a rating (organizer only - can only delete their own rating)
pub async fn delete_rating(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Delete the rating
    let result = sqlx::query(
        r#"
        DELETE FROM ratings
        WHERE talk_id = $1 AND organizer_id = $2
        "#,
    )
    .bind(talk_id)
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error deleting rating: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to delete rating")),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Rating not found")),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Get the current user's rating for a talk (organizer only)
pub async fn get_my_rating(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(talk_id): Path<Uuid>,
) -> Result<Json<RatingResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the rating
    let rating = sqlx::query_as::<_, Rating>(
        r#"
        SELECT *
        FROM ratings
        WHERE talk_id = $1 AND organizer_id = $2
        "#,
    )
    .bind(talk_id)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching rating: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch rating")),
        )
    })?;

    match rating {
        Some(rating) => {
            let response = RatingResponse::from(rating)
                .with_organizer_info(user.full_name.clone(), user.email.clone());
            Ok(Json(response))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Rating not found")),
        )),
    }
}

/// Get aggregated ratings statistics for all talks (organizer only)
pub async fn get_ratings_statistics(
    State(state): State<AppState>,
) -> Result<Json<RatingsStatisticsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get total talks count
    let total_talks = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM talks
        "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error counting talks: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch statistics")),
        )
    })?;

    // Get total ratings count
    let total_ratings = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM ratings
        "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error counting ratings: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch statistics")),
        )
    })?;

    // Get rating distribution
    #[derive(sqlx::FromRow)]
    struct DistributionRow {
        rating: i32,
        count: i64,
    }

    let distribution_rows = sqlx::query_as::<_, DistributionRow>(
        r#"
        SELECT rating, COUNT(*) as count
        FROM ratings
        GROUP BY rating
        ORDER BY rating
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching distribution: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch statistics")),
        )
    })?;

    let mut rating_distribution = RatingDistribution {
        one_star: 0,
        two_star: 0,
        three_star: 0,
        four_star: 0,
        five_star: 0,
    };

    for row in distribution_rows {
        match row.rating {
            1 => rating_distribution.one_star = row.count,
            2 => rating_distribution.two_star = row.count,
            3 => rating_distribution.three_star = row.count,
            4 => rating_distribution.four_star = row.count,
            5 => rating_distribution.five_star = row.count,
            _ => {}
        }
    }

    // Get talk statistics with ratings
    #[derive(sqlx::FromRow)]
    struct TalkStatsRow {
        talk_id: Uuid,
        talk_title: String,
        speaker_name: String,
        state: String,
        rating_count: i64,
        rating_sum: Option<i64>,
        ratings_json: Option<String>,
    }

    let talk_stats_rows = sqlx::query_as::<_, TalkStatsRow>(
        r#"
        SELECT
            t.id as talk_id,
            t.title as talk_title,
            u.full_name as speaker_name,
            t.state::text as state,
            COUNT(r.id) as rating_count,
            SUM(r.rating) as rating_sum,
            COALESCE(
                json_agg(r.rating ORDER BY r.created_at DESC)
                FILTER (WHERE r.id IS NOT NULL),
                '[]'
            )::text as ratings_json
        FROM talks t
        JOIN users u ON t.speaker_id = u.id
        LEFT JOIN ratings r ON t.id = r.talk_id
        GROUP BY t.id, t.title, u.full_name, t.state
        ORDER BY rating_count DESC, rating_sum DESC NULLS LAST, t.title
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching talk stats: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch statistics")),
        )
    })?;

    let mut talk_stats = Vec::new();
    let mut talks_with_ratings = 0i64;
    let mut total_rating_sum = 0i64;

    for row in talk_stats_rows {
        let average_rating = if row.rating_count > 0 {
            talks_with_ratings += 1;
            let sum = row.rating_sum.unwrap_or(0);
            total_rating_sum += sum;
            Some(sum as f64 / row.rating_count as f64)
        } else {
            None
        };

        let ratings: Vec<i32> = row
            .ratings_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        talk_stats.push(TalkRatingStats {
            talk_id: row.talk_id,
            talk_title: row.talk_title,
            speaker_name: row.speaker_name,
            state: row.state,
            average_rating,
            rating_count: row.rating_count,
            ratings,
        });
    }

    let overall_average_rating = if total_ratings > 0 {
        Some(total_rating_sum as f64 / total_ratings as f64)
    } else {
        None
    };

    let talks_without_ratings = total_talks - talks_with_ratings;

    Ok(Json(RatingsStatisticsResponse {
        total_talks,
        total_ratings,
        talks_with_ratings,
        talks_without_ratings,
        overall_average_rating,
        rating_distribution,
        talk_stats,
    }))
}
