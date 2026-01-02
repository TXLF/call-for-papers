use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::ErrorResponse, AssignTalkRequest, CreateScheduleSlotRequest, PublicScheduleSlot,
        PublicScheduleTalk, ScheduleSlot, ScheduleSlotResponse, UpdateScheduleSlotRequest,
    },
};

/// List all schedule slots (public endpoint)
pub async fn list_schedule_slots(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScheduleSlotResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let slots = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        SELECT * FROM schedule_slots
        ORDER BY slot_date ASC, start_time ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching schedule slots: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch schedule slots")),
        )
    })?;

    let responses: Vec<ScheduleSlotResponse> =
        slots.into_iter().map(ScheduleSlotResponse::from).collect();
    Ok(Json(responses))
}

/// Get a single schedule slot by ID (public endpoint)
pub async fn get_schedule_slot(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
) -> Result<Json<ScheduleSlotResponse>, (StatusCode, Json<ErrorResponse>)> {
    let slot = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        SELECT * FROM schedule_slots
        WHERE id = $1
        "#,
    )
    .bind(slot_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching schedule slot: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch schedule slot")),
        )
    })?;

    let slot = slot.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Schedule slot not found")),
        )
    })?;

    Ok(Json(ScheduleSlotResponse::from(slot)))
}

/// Create a new schedule slot (organizer only)
pub async fn create_schedule_slot(
    State(state): State<AppState>,
    Json(payload): Json<CreateScheduleSlotRequest>,
) -> Result<(StatusCode, Json<ScheduleSlotResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate time order
    if payload.start_time >= payload.end_time {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Start time must be before end time")),
        ));
    }

    // Create the schedule slot
    let slot = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        INSERT INTO schedule_slots (conference_id, track_id, slot_date, start_time, end_time)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(payload.conference_id)
    .bind(payload.track_id)
    .bind(payload.slot_date)
    .bind(payload.start_time)
    .bind(payload.end_time)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating schedule slot: {}", e);
        // Check for foreign key violation
        if let Some(db_err) = e.as_database_error() {
            if db_err.message().contains("foreign key") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("Invalid conference ID or track ID")),
                );
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create schedule slot")),
        )
    })?;

    tracing::info!(
        "Schedule slot created: {} on {} from {} to {}",
        slot.id,
        slot.slot_date,
        slot.start_time,
        slot.end_time
    );
    Ok((StatusCode::CREATED, Json(ScheduleSlotResponse::from(slot))))
}

/// Update a schedule slot (organizer only)
pub async fn update_schedule_slot(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(payload): Json<UpdateScheduleSlotRequest>,
) -> Result<Json<ScheduleSlotResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the existing slot
    let existing_slot = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        SELECT * FROM schedule_slots
        WHERE id = $1
        "#,
    )
    .bind(slot_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching schedule slot: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch schedule slot")),
        )
    })?;

    let existing_slot = existing_slot.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Schedule slot not found")),
        )
    })?;

    // Prepare updated values
    let track_id = payload.track_id.unwrap_or(existing_slot.track_id);
    let talk_id = payload.talk_id.or(existing_slot.talk_id);
    let slot_date = payload.slot_date.unwrap_or(existing_slot.slot_date);
    let start_time = payload.start_time.unwrap_or(existing_slot.start_time);
    let end_time = payload.end_time.unwrap_or(existing_slot.end_time);

    // Validate time order
    if start_time >= end_time {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Start time must be before end time")),
        ));
    }

    // Update the slot
    let updated_slot = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        UPDATE schedule_slots
        SET track_id = $1, talk_id = $2, slot_date = $3, start_time = $4, end_time = $5, updated_at = $6
        WHERE id = $7
        RETURNING *
        "#,
    )
    .bind(track_id)
    .bind(talk_id)
    .bind(slot_date)
    .bind(start_time)
    .bind(end_time)
    .bind(Utc::now())
    .bind(slot_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating schedule slot: {}", e);
        // Check for foreign key violation
        if let Some(db_err) = e.as_database_error() {
            if db_err.message().contains("foreign key") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("Invalid track ID or talk ID")),
                );
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to update schedule slot")),
        )
    })?;

    tracing::info!("Schedule slot updated: {}", updated_slot.id);
    Ok(Json(ScheduleSlotResponse::from(updated_slot)))
}

/// Delete a schedule slot (organizer only)
pub async fn delete_schedule_slot(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query(
        r#"
        DELETE FROM schedule_slots
        WHERE id = $1
        "#,
    )
    .bind(slot_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error deleting schedule slot: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to delete schedule slot")),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Schedule slot not found")),
        ));
    }

    tracing::info!("Schedule slot deleted: {}", slot_id);
    Ok(StatusCode::NO_CONTENT)
}

/// Assign a talk to a schedule slot (organizer only)
pub async fn assign_talk_to_slot(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
    Json(payload): Json<AssignTalkRequest>,
) -> Result<Json<ScheduleSlotResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify the slot exists
    let existing_slot = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        SELECT * FROM schedule_slots
        WHERE id = $1
        "#,
    )
    .bind(slot_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching schedule slot: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch schedule slot")),
        )
    })?;

    if existing_slot.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Schedule slot not found")),
        ));
    }

    // Update the slot with the talk assignment
    let updated_slot = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        UPDATE schedule_slots
        SET talk_id = $1, updated_at = $2
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(payload.talk_id)
    .bind(Utc::now())
    .bind(slot_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error assigning talk to slot: {}", e);
        // Check for foreign key violation (invalid talk_id)
        if let Some(db_err) = e.as_database_error() {
            if db_err.message().contains("foreign key") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("Invalid talk ID")),
                );
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to assign talk to slot")),
        )
    })?;

    tracing::info!(
        "Talk {} assigned to schedule slot {}",
        payload.talk_id,
        slot_id
    );
    Ok(Json(ScheduleSlotResponse::from(updated_slot)))
}

/// Unassign a talk from a schedule slot (organizer only)
pub async fn unassign_talk_from_slot(
    State(state): State<AppState>,
    Path(slot_id): Path<Uuid>,
) -> Result<Json<ScheduleSlotResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Update the slot to remove talk assignment
    let updated_slot = sqlx::query_as::<_, ScheduleSlot>(
        r#"
        UPDATE schedule_slots
        SET talk_id = NULL, updated_at = $1
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(Utc::now())
    .bind(slot_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error unassigning talk from slot: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to unassign talk from slot")),
        )
    })?;

    let updated_slot = updated_slot.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Schedule slot not found")),
        )
    })?;

    tracing::info!("Talk unassigned from schedule slot {}", slot_id);
    Ok(Json(ScheduleSlotResponse::from(updated_slot)))
}

/// Get public schedule with talk details (public endpoint)
pub async fn get_public_schedule(
    State(state): State<AppState>,
) -> Result<Json<Vec<PublicScheduleSlot>>, (StatusCode, Json<ErrorResponse>)> {
    // Query that joins schedule_slots, tracks, and talks
    // Using sqlx::query instead of query! macro to avoid compile-time type inference issues
    let rows = sqlx::query(
        r#"
        SELECT
            ss.id,
            ss.track_id,
            t.name as track_name,
            ss.slot_date,
            ss.start_time,
            ss.end_time,
            ss.talk_id,
            tk.title as talk_title,
            tk.short_summary as talk_summary,
            u.full_name as speaker_name
        FROM schedule_slots ss
        INNER JOIN tracks t ON ss.track_id = t.id
        LEFT JOIN talks tk ON ss.talk_id = tk.id
        LEFT JOIN users u ON tk.speaker_id = u.id
        ORDER BY ss.slot_date ASC, ss.start_time ASC, t.name ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching public schedule: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch schedule")),
        )
    })?;

    let mut schedule = Vec::new();
    for row in rows {
        let slot_id: Uuid = row.try_get("id").map_err(|e| {
            tracing::error!("Error getting slot id: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to parse schedule data")),
            )
        })?;

        let track_id: Uuid = row.try_get("track_id").map_err(|e| {
            tracing::error!("Error getting track_id: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to parse schedule data")),
            )
        })?;

        let track_name: String = row.try_get("track_name").map_err(|e| {
            tracing::error!("Error getting track_name: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to parse schedule data")),
            )
        })?;

        let slot_date = row.try_get("slot_date").map_err(|e| {
            tracing::error!("Error getting slot_date: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to parse schedule data")),
            )
        })?;

        let start_time = row.try_get("start_time").map_err(|e| {
            tracing::error!("Error getting start_time: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to parse schedule data")),
            )
        })?;

        let end_time = row.try_get("end_time").map_err(|e| {
            tracing::error!("Error getting end_time: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to parse schedule data")),
            )
        })?;

        // Optional talk fields (LEFT JOIN)
        let talk_id: Option<Uuid> = row.try_get("talk_id").ok();
        let talk_title: Option<String> = row.try_get("talk_title").ok();
        let talk_summary: Option<String> = row.try_get("talk_summary").ok();
        let speaker_name: Option<String> = row.try_get("speaker_name").ok();

        let talk = if let (Some(id), Some(title), Some(summary), Some(speaker)) =
            (talk_id, talk_title, talk_summary, speaker_name)
        {
            Some(PublicScheduleTalk {
                id,
                title,
                short_summary: summary,
                speaker_name: speaker,
            })
        } else {
            None
        };

        schedule.push(PublicScheduleSlot {
            id: slot_id,
            track_id,
            track_name,
            slot_date,
            start_time,
            end_time,
            talk,
        });
    }

    Ok(Json(schedule))
}
