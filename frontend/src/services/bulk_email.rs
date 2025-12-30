use gloo_net::http::Request;
use crate::{
    services::auth::AuthService,
    types::{BulkEmailRequest, BulkEmailResponse, ErrorResponse},
};

pub struct BulkEmailService;

impl BulkEmailService {
    /// Send bulk email to filtered recipients (organizer only)
    pub async fn send_bulk_email(request: BulkEmailRequest) -> Result<BulkEmailResponse, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::post("/api/bulk-email")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<BulkEmailResponse>()
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
}
