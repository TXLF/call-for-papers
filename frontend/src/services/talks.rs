use gloo_net::http::Request;

use crate::{
    services::auth::AuthService,
    types::{CreateTalkRequest, ErrorResponse, Talk, UpdateTalkRequest},
};

pub struct TalkService;

impl TalkService {
    pub async fn create_talk(request: CreateTalkRequest) -> Result<Talk, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::post("/api/talks")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let talk = response
                .json::<Talk>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(talk)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn get_my_talks() -> Result<Vec<Talk>, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get("/api/talks/mine")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let talks = response
                .json::<Vec<Talk>>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(talks)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn get_talk(id: &str) -> Result<Talk, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get(&format!("/api/talks/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let talk = response
                .json::<Talk>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(talk)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn update_talk(id: &str, request: UpdateTalkRequest) -> Result<Talk, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::put(&format!("/api/talks/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let talk = response
                .json::<Talk>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(talk)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn delete_talk(id: &str) -> Result<(), String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/talks/{}", id))
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

    pub async fn upload_slides(id: &str, file: web_sys::File) -> Result<Talk, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        // Create FormData
        let form_data = web_sys::FormData::new()
            .map_err(|e| format!("Failed to create FormData: {:?}", e))?;

        form_data
            .append_with_blob("slides", &file)
            .map_err(|e| format!("Failed to append file: {:?}", e))?;

        let response = Request::post(&format!("/api/talks/{}/upload-slides", id))
            .header("Authorization", &format!("Bearer {}", token))
            .body(form_data)
            .map_err(|e| format!("Failed to create request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let talk = response
                .json::<Talk>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(talk)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn respond_to_talk(id: &str, action: &str) -> Result<Talk, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let request = crate::types::RespondToTalkRequest {
            action: action.to_string(),
        };

        let response = Request::post(&format!("/api/talks/{}/respond", id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let talk = response
                .json::<Talk>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(talk)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// List all talks (organizer-only) with optional state filtering
    pub async fn list_all_talks(state_filter: Option<String>) -> Result<Vec<Talk>, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        // Build URL with optional query parameter
        let url = if let Some(state) = state_filter {
            format!("/api/talks?state={}", state)
        } else {
            "/api/talks".to_string()
        };

        let response = Request::get(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let talks = response
                .json::<Vec<Talk>>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(talks)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }
}
