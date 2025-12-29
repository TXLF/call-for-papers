use gloo_net::http::Request;
use crate::{services::auth::AuthService, types::{Track, ErrorResponse, CreateTrackRequest, UpdateTrackRequest}};

pub struct TrackService;

impl TrackService {
    /// List all tracks (public endpoint, no auth required)
    pub async fn list_tracks() -> Result<Vec<Track>, String> {
        let response = Request::get("/api/tracks")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<Track>>()
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

    /// Get a single track by ID (public endpoint)
    pub async fn get_track(id: &str) -> Result<Track, String> {
        let response = Request::get(&format!("/api/tracks/{}", id))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Track>()
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

    /// Create a new track (organizer only)
    pub async fn create_track(request: CreateTrackRequest) -> Result<Track, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::post("/api/tracks")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Track>()
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

    /// Update an existing track (organizer only)
    pub async fn update_track(id: &str, request: UpdateTrackRequest) -> Result<Track, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::put(&format!("/api/tracks/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Track>()
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

    /// Delete a track (organizer only)
    pub async fn delete_track(id: &str) -> Result<(), String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/tracks/{}", id))
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
