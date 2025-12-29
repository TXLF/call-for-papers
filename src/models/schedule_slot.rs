use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduleSlot {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub track_id: Uuid,
    pub talk_id: Option<Uuid>,
    pub slot_date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleSlotRequest {
    pub conference_id: Uuid,
    pub track_id: Uuid,
    pub slot_date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleSlotRequest {
    pub track_id: Option<Uuid>,
    pub talk_id: Option<Uuid>,
    pub slot_date: Option<NaiveDate>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTalkRequest {
    pub talk_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ScheduleSlotResponse {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub track_id: Uuid,
    pub talk_id: Option<Uuid>,
    pub slot_date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ScheduleSlot> for ScheduleSlotResponse {
    fn from(slot: ScheduleSlot) -> Self {
        Self {
            id: slot.id,
            conference_id: slot.conference_id,
            track_id: slot.track_id,
            talk_id: slot.talk_id,
            slot_date: slot.slot_date,
            start_time: slot.start_time,
            end_time: slot.end_time,
            created_at: slot.created_at,
            updated_at: slot.updated_at,
        }
    }
}
