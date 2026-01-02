use handlebars::Handlebars;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmailTemplate {
    pub id: Uuid,
    pub conference_id: Uuid,
    pub template_type: String,
    pub subject: String,
    pub body: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVariables {
    pub speaker_name: String,
    pub speaker_email: String,
    pub talk_title: String,
    pub talk_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_name: Option<String>,
}

#[derive(Clone)]
pub struct EmailService {
    config: Config,
    db: PgPool,
    handlebars: Arc<Handlebars<'static>>,
}

impl EmailService {
    pub fn new(config: Config, db: PgPool) -> Self {
        let handlebars = Arc::new(Handlebars::new());
        Self {
            config,
            db,
            handlebars,
        }
    }

    /// Check if email is configured
    pub fn is_configured(&self) -> bool {
        self.config.smtp_host.is_some()
            && self.config.smtp_user.is_some()
            && self.config.smtp_password.is_some()
            && self.config.smtp_from.is_some()
    }

    /// Get email template by type for a conference
    pub async fn get_template(
        &self,
        conference_id: Uuid,
        template_type: &str,
    ) -> Result<EmailTemplate, String> {
        // Try to get conference-specific template first
        if let Ok(template) = sqlx::query_as::<_, EmailTemplate>(
            r#"
            SELECT * FROM email_templates
            WHERE conference_id = $1 AND template_type = $2
            ORDER BY is_default DESC, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(conference_id)
        .bind(template_type)
        .fetch_one(&self.db)
        .await
        {
            return Ok(template);
        }

        // Fall back to default template
        sqlx::query_as::<_, EmailTemplate>(
            r#"
            SELECT * FROM email_templates
            WHERE is_default = true AND template_type = $1
            LIMIT 1
            "#,
        )
        .bind(template_type)
        .fetch_one(&self.db)
        .await
        .map_err(|e| format!("No template found for type {}: {}", template_type, e))
    }

    /// Render template with variables
    pub fn render_template(
        &self,
        template: &str,
        variables: &EmailVariables,
    ) -> Result<String, String> {
        // Convert EmailVariables to HashMap for handlebars
        let json_value = serde_json::to_value(variables).map_err(|e| e.to_string())?;
        let vars: HashMap<String, serde_json::Value> = if let Some(obj) = json_value.as_object() {
            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            HashMap::new()
        };

        self.handlebars
            .render_template(template, &vars)
            .map_err(|e| format!("Template rendering error: {}", e))
    }

    /// Send an email using SMTP
    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
        template_id: Option<Uuid>,
        talk_id: Option<Uuid>,
        sent_by: Option<Uuid>,
    ) -> Result<(), String> {
        if !self.is_configured() {
            tracing::warn!("Email not configured, skipping email send to {}", to_email);
            return Ok(());
        }

        let smtp_host = self.config.smtp_host.as_ref().unwrap();
        let smtp_port = self.config.smtp_port.unwrap_or(587);
        let smtp_user = self.config.smtp_user.as_ref().unwrap();
        let smtp_password = self.config.smtp_password.as_ref().unwrap();
        let smtp_from = self.config.smtp_from.as_ref().unwrap();

        // Build email message
        let email = Message::builder()
            .from(
                smtp_from
                    .parse()
                    .map_err(|e| format!("Invalid from address: {}", e))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| format!("Invalid to address: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {}", e))?;

        // Setup SMTP transport
        let creds = Credentials::new(smtp_user.clone(), smtp_password.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_host)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .port(smtp_port)
            .credentials(creds)
            .build();

        // Send email
        mailer
            .send(email)
            .await
            .map_err(|e| format!("Failed to send email: {}", e))?;

        // Log email to database
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO email_logs (recipient_email, subject, body, template_id, talk_id, sent_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(to_email)
        .bind(subject)
        .bind(body)
        .bind(template_id)
        .bind(talk_id)
        .bind(sent_by)
        .execute(&self.db)
        .await
        {
            tracing::error!("Failed to log email to database: {}", e);
        }

        tracing::info!("Email sent successfully to {}", to_email);
        Ok(())
    }

    /// Send email using template
    pub async fn send_templated_email(
        &self,
        conference_id: Uuid,
        template_type: &str,
        to_email: &str,
        variables: EmailVariables,
        talk_id: Option<Uuid>,
        sent_by: Option<Uuid>,
    ) -> Result<(), String> {
        let template = self.get_template(conference_id, template_type).await?;

        let subject = self.render_template(&template.subject, &variables)?;
        let body = self.render_template(&template.body, &variables)?;

        self.send_email(
            to_email,
            &subject,
            &body,
            Some(template.id),
            talk_id,
            sent_by,
        )
        .await
    }
}
