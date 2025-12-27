use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "talk_state", rename_all = "lowercase")]
pub enum TalkState {
    Submitted,
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Talk {
    pub id: Uuid,
    pub speaker_id: Uuid,
    pub title: String,
    pub short_summary: String,
    pub long_description: Option<String>,
    pub slides_url: Option<String>,
    pub state: TalkState,
    pub submitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTalkRequest {
    pub title: String,
    pub short_summary: String,
    pub long_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTalkRequest {
    pub title: Option<String>,
    pub short_summary: Option<String>,
    pub long_description: Option<String>,
    pub slides_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TalkResponse {
    pub id: Uuid,
    pub speaker_id: Uuid,
    pub title: String,
    pub short_summary: String,
    pub long_description: Option<String>,
    pub slides_url: Option<String>,
    pub state: TalkState,
    pub submitted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Talk> for TalkResponse {
    fn from(talk: Talk) -> Self {
        Self {
            id: talk.id,
            speaker_id: talk.speaker_id,
            title: talk.title,
            short_summary: talk.short_summary,
            long_description: talk.long_description,
            slides_url: talk.slides_url,
            state: talk.state,
            submitted_at: talk.submitted_at,
            updated_at: talk.updated_at,
        }
    }
}
