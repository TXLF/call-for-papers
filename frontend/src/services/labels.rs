use gloo_net::http::Request;
use crate::{services::auth::AuthService, types::{Label, ErrorResponse, AddLabelToTalkRequest, CreateLabelRequest, UpdateLabelRequest}};

pub struct LabelService;

impl LabelService {
    /// List all labels (public endpoint, no auth required)
    pub async fn list_labels() -> Result<Vec<Label>, String> {
        let response = Request::get("/api/labels")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<Label>>()
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

    /// Get labels for a specific talk
    pub async fn get_talk_labels(talk_id: &str) -> Result<Vec<Label>, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get(&format!("/api/talks/{}/labels", talk_id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<Label>>()
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

    /// Add labels to a talk
    pub async fn add_labels_to_talk(talk_id: &str, label_ids: Vec<String>) -> Result<Vec<Label>, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let request = AddLabelToTalkRequest { label_ids };

        let response = Request::post(&format!("/api/talks/{}/labels", talk_id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<Label>>()
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

    /// Remove a label from a talk
    pub async fn remove_label_from_talk(talk_id: &str, label_id: &str) -> Result<(), String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/talks/{}/labels/{}", talk_id, label_id))
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

    /// Create a new label (organizer only)
    pub async fn create_label(request: CreateLabelRequest) -> Result<Label, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::post("/api/labels")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Label>()
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

    /// Update an existing label (organizer only)
    pub async fn update_label(id: &str, request: UpdateLabelRequest) -> Result<Label, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::put(&format!("/api/labels/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Label>()
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

    /// Delete a label (organizer only)
    pub async fn delete_label(id: &str) -> Result<(), String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/labels/{}", id))
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
