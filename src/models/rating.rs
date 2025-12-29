use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Rating {
    pub id: Uuid,
    pub talk_id: Uuid,
    pub organizer_id: Uuid,
    pub rating: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRatingRequest {
    pub rating: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRatingRequest {
    pub rating: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RatingResponse {
    pub id: Uuid,
    pub talk_id: Uuid,
    pub organizer_id: Uuid,
    pub organizer_name: String,
    pub organizer_email: String,
    pub rating: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Rating> for RatingResponse {
    fn from(rating: Rating) -> Self {
        Self {
            id: rating.id,
            talk_id: rating.talk_id,
            organizer_id: rating.organizer_id,
            organizer_name: String::new(), // Will be populated by handlers
            organizer_email: String::new(), // Will be populated by handlers
            rating: rating.rating,
            notes: rating.notes,
            created_at: rating.created_at,
            updated_at: rating.updated_at,
        }
    }
}

impl RatingResponse {
    pub fn with_organizer_info(mut self, name: String, email: String) -> Self {
        self.organizer_name = name;
        self.organizer_email = email;
        self
    }
}
