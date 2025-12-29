use gloo_net::http::Request;

use crate::{
    services::auth::AuthService,
    types::{CreateRatingRequest, ErrorResponse, Rating},
};

pub struct RatingService;

impl RatingService {
    /// Create or update a rating for a talk (organizer only)
    pub async fn create_or_update_rating(
        talk_id: &str,
        rating: i32,
        notes: Option<String>,
    ) -> Result<Rating, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let request_body = CreateRatingRequest { rating, notes };

        let response = Request::post(&format!("/api/talks/{}/rate", talk_id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request_body)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let rating = response
                .json::<Rating>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(rating)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// Get all ratings for a talk (organizer only)
    pub async fn get_talk_ratings(talk_id: &str) -> Result<Vec<Rating>, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get(&format!("/api/talks/{}/ratings", talk_id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let ratings = response
                .json::<Vec<Rating>>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(ratings)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// Get the current user's rating for a talk (organizer only)
    pub async fn get_my_rating(talk_id: &str) -> Result<Option<Rating>, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::get(&format!("/api/talks/{}/rate/mine", talk_id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let rating = response
                .json::<Rating>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(Some(rating))
        } else if response.status() == 404 {
            // No rating found for this user
            Ok(None)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    /// Delete the current user's rating for a talk (organizer only)
    pub async fn delete_rating(talk_id: &str) -> Result<(), String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/talks/{}/rate", talk_id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() || response.status() == 204 {
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
