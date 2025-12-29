use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: Option<String>,
    pub full_name: String,
    pub bio: Option<String>,
    pub is_organizer: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: Option<String>,
    pub password: String,
    pub full_name: String,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TalkState {
    Submitted,
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_ai_generated: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateLabelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Talk {
    pub id: String,
    pub speaker_id: String,
    pub title: String,
    pub short_summary: String,
    pub long_description: Option<String>,
    pub slides_url: Option<String>,
    pub state: TalkState,
    pub submitted_at: String,
    pub updated_at: String,
    pub labels: Vec<Label>,
    pub speaker_name: String,
    pub speaker_email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTalkRequest {
    pub title: String,
    pub short_summary: String,
    pub long_description: Option<String>,
    pub label_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTalkRequest {
    pub title: Option<String>,
    pub short_summary: Option<String>,
    pub long_description: Option<String>,
    pub slides_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RespondToTalkRequest {
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct ChangeStateRequest {
    pub new_state: TalkState,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddLabelToTalkRequest {
    pub label_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rating {
    pub id: String,
    pub talk_id: String,
    pub organizer_id: String,
    pub organizer_name: String,
    pub organizer_email: String,
    pub rating: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateRatingRequest {
    pub rating: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TalkRatingStats {
    pub talk_id: String,
    pub talk_title: String,
    pub speaker_name: String,
    pub state: String,
    pub average_rating: Option<f64>,
    pub rating_count: i64,
    pub ratings: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RatingsStatisticsResponse {
    pub total_talks: i64,
    pub total_ratings: i64,
    pub talks_with_ratings: i64,
    pub talks_without_ratings: i64,
    pub overall_average_rating: Option<f64>,
    pub rating_distribution: RatingDistribution,
    pub talk_stats: Vec<TalkRatingStats>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RatingDistribution {
    pub one_star: i64,
    pub two_star: i64,
    pub three_star: i64,
    pub four_star: i64,
    pub five_star: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DashboardStats {
    pub total_talks: i64,
    pub talks_by_state: TalksByState,
    pub rating_stats: RatingStats,
    pub recent_submissions: Vec<RecentTalk>,
    pub unrated_talks: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TalksByState {
    pub submitted: i64,
    pub pending: i64,
    pub accepted: i64,
    pub rejected: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RatingStats {
    pub total_ratings: i64,
    pub average_rating: Option<f64>,
    pub talks_with_ratings: i64,
    pub talks_without_ratings: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RecentTalk {
    pub id: String,
    pub title: String,
    pub speaker_name: String,
    pub state: TalkState,
    pub submitted_at: String,
    pub rating_count: Option<i64>,
    pub average_rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub id: String,
    pub conference_id: String,
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTrackRequest {
    pub conference_id: String,
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTrackRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduleSlot {
    pub id: String,
    pub conference_id: String,
    pub track_id: String,
    pub talk_id: Option<String>,
    pub slot_date: String,
    pub start_time: String,
    pub end_time: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateScheduleSlotRequest {
    pub conference_id: String,
    pub track_id: String,
    pub slot_date: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateScheduleSlotRequest {
    pub track_id: Option<String>,
    pub talk_id: Option<String>,
    pub slot_date: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AssignTalkRequest {
    pub talk_id: String,
}
