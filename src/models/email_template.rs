use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailTemplate {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub template_type: String,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmailTemplateRequest {
    pub conference_id: Uuid,
    pub template_type: String,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmailTemplateRequest {
    pub name: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct EmailTemplateResponse {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub template_type: String,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmailTemplate> for EmailTemplateResponse {
    fn from(template: EmailTemplate) -> Self {
        Self {
            id: template.id,
            conference_id: template.conference_id,
            template_type: template.template_type,
            name: template.name,
            subject: template.subject,
            body: template.body,
            is_default: template.is_default,
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }
}
