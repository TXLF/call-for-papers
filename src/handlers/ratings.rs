use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, CreateRatingRequest, Rating, RatingResponse, User,
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

    let response = RatingResponse::from(rating).with_organizer_info(
        user.full_name.clone(),
        user.email.clone(),
    );

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

    // Fetch all ratings for the talk
    let ratings = sqlx::query_as::<_, Rating>(
        r#"
        SELECT id, talk_id, organizer_id, rating, notes, created_at, updated_at
        FROM ratings
        WHERE talk_id = $1
        ORDER BY created_at DESC
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

    // Fetch organizer information for each rating
    let mut responses = Vec::new();
    for rating in ratings {
        let organizer = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(rating.organizer_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching organizer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to fetch organizer information")),
            )
        })?;

        responses.push(
            RatingResponse::from(rating).with_organizer_info(
                organizer.full_name,
                organizer.email,
            )
        );
    }

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
            let response = RatingResponse::from(rating).with_organizer_info(
                user.full_name.clone(),
                user.email.clone(),
            );
            Ok(Json(response))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Rating not found")),
        )),
    }
}
