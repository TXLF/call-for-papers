use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Label {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLabelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Label> for LabelResponse {
    fn from(label: Label) -> Self {
        Self {
            id: label.id,
            name: label.name,
            description: label.description,
            color: label.color,
            is_ai_generated: label.is_ai_generated,
            created_at: label.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddLabelToTalkRequest {
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TalkLabel {
    pub talk_id: Uuid,
    pub label_id: Uuid,
    pub added_by: Option<Uuid>,
    pub added_at: DateTime<Utc>,
}
