use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Track {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrackRequest {
    pub conference_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TrackResponse {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<Track> for TrackResponse {
    fn from(track: Track) -> Self {
        Self {
            id: track.id,
            conference_id: track.conference_id,
            name: track.name,
            description: track.description,
            capacity: track.capacity,
            created_at: track.created_at,
        }
    }
}
