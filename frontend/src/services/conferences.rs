use gloo_net::http::Request;
use crate::types::{Conference, ErrorResponse};

pub struct ConferenceService;

impl ConferenceService {
    /// Get the active conference (public endpoint)
    pub async fn get_active_conference() -> Result<Conference, String> {
        let response = Request::get("/api/conferences/active")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Conference>()
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
