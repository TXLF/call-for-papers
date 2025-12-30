use gloo_net::http::Request;
use crate::{
    services::auth::AuthService,
    types::{EmailTemplate, ErrorResponse, CreateEmailTemplateRequest, UpdateEmailTemplateRequest},
};

pub struct EmailTemplateService;

impl EmailTemplateService {
    /// List all email templates (organizer only)
    pub async fn list_email_templates() -> Result<Vec<EmailTemplate>, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get("/api/email-templates")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<EmailTemplate>>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// Get a single email template by ID (organizer only)
    pub async fn get_email_template(id: &str) -> Result<EmailTemplate, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get(&format!("/api/email-templates/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<EmailTemplate>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// Create a new email template (organizer only)
    pub async fn create_email_template(request: CreateEmailTemplateRequest) -> Result<EmailTemplate, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::post("/api/email-templates")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<EmailTemplate>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// Update an existing email template (organizer only)
    pub async fn update_email_template(id: &str, request: UpdateEmailTemplateRequest) -> Result<EmailTemplate, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::put(&format!("/api/email-templates/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<EmailTemplate>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// Delete an email template (organizer only)
    pub async fn delete_email_template(id: &str) -> Result<(), String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/email-templates/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            Ok(())
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }
}
