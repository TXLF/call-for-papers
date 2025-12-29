use gloo_net::http::Request;

use crate::{
    services::auth::AuthService,
    types::{DashboardStats, ErrorResponse},
};

pub struct DashboardService;

impl DashboardService {
    /// Get dashboard statistics (organizer only)
    pub async fn get_stats() -> Result<DashboardStats, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get("/api/dashboard/stats")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let stats = response
                .json::<DashboardStats>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(stats)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }
}
