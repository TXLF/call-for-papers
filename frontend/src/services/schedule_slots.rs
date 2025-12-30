use gloo_net::http::Request;
use crate::{services::auth::AuthService, types::{ScheduleSlot, ErrorResponse, CreateScheduleSlotRequest, UpdateScheduleSlotRequest, AssignTalkRequest, PublicScheduleSlot}};

pub struct ScheduleSlotService;

impl ScheduleSlotService {
    /// Get public schedule with talk details (public endpoint, no auth required)
    pub async fn get_public_schedule() -> Result<Vec<PublicScheduleSlot>, String> {
        let response = Request::get("/api/schedule")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<PublicScheduleSlot>>()
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

    /// List all schedule slots (public endpoint, no auth required)
    pub async fn list_schedule_slots() -> Result<Vec<ScheduleSlot>, String> {
        let response = Request::get("/api/schedule-slots")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<ScheduleSlot>>()
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

    /// Get a single schedule slot by ID (public endpoint)
    pub async fn get_schedule_slot(id: &str) -> Result<ScheduleSlot, String> {
        let response = Request::get(&format!("/api/schedule-slots/{}", id))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<ScheduleSlot>()
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

    /// Create a new schedule slot (organizer only)
    pub async fn create_schedule_slot(request: CreateScheduleSlotRequest) -> Result<ScheduleSlot, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::post("/api/schedule-slots")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<ScheduleSlot>()
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

    /// Update an existing schedule slot (organizer only)
    pub async fn update_schedule_slot(id: &str, request: UpdateScheduleSlotRequest) -> Result<ScheduleSlot, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::put(&format!("/api/schedule-slots/{}", id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<ScheduleSlot>()
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

    /// Delete a schedule slot (organizer only)
    pub async fn delete_schedule_slot(id: &str) -> Result<(), String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/schedule-slots/{}", id))
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

    /// Assign a talk to a schedule slot (organizer only)
    pub async fn assign_talk_to_slot(slot_id: &str, request: AssignTalkRequest) -> Result<ScheduleSlot, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::put(&format!("/api/schedule-slots/{}/assign", slot_id))
            .header("Authorization", &format!("Bearer {}", token))
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<ScheduleSlot>()
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

    /// Unassign a talk from a schedule slot (organizer only)
    pub async fn unassign_talk_from_slot(slot_id: &str) -> Result<ScheduleSlot, String> {
        let token = AuthService::get_token().ok_or("Not authenticated")?;

        let response = Request::delete(&format!("/api/schedule-slots/{}/assign", slot_id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<ScheduleSlot>()
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
