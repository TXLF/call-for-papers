use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConferenceConfig {
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub year: u16,
    pub website: String,
    pub submission_open: String,
    pub submission_close: String,
    pub conference_start: String,
    pub conference_end: String,
    pub location: String,
    pub venue: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrandingConfig {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub logo_light: String,
    pub logo_dark: String,
    pub favicon: String,
    #[serde(default)]
    pub custom_css: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeaturesConfig {
    #[serde(default = "default_true")]
    pub enable_speaker_registration: bool,
    #[serde(default)]
    pub enable_organizer_registration: bool,
    #[serde(default = "default_true")]
    pub enable_social_login: bool,
    #[serde(default = "default_true")]
    pub enable_google_login: bool,
    #[serde(default = "default_true")]
    pub enable_github_login: bool,
    #[serde(default)]
    pub enable_apple_login: bool,
    #[serde(default)]
    pub enable_facebook_login: bool,
    #[serde(default)]
    pub enable_linkedin_login: bool,
    #[serde(default = "default_true")]
    pub enable_slide_upload: bool,
    #[serde(default = "default_true")]
    pub enable_ratings: bool,
    #[serde(default = "default_true")]
    pub enable_ai_tagging: bool,
    #[serde(default = "default_true")]
    pub enable_schedule_builder: bool,
    #[serde(default = "default_true")]
    pub require_speaker_confirmation: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubmissionConfig {
    #[serde(default = "default_min_title_length")]
    pub min_title_length: usize,
    #[serde(default = "default_max_title_length")]
    pub max_title_length: usize,
    #[serde(default = "default_min_summary_length")]
    pub min_summary_length: usize,
    #[serde(default = "default_max_summary_length")]
    pub max_summary_length: usize,
    #[serde(default = "default_max_description_length")]
    pub max_description_length: usize,
    #[serde(default = "default_max_slide_size")]
    pub max_slide_size_mb: u64,
    #[serde(default = "default_slide_formats")]
    pub allowed_slide_formats: Vec<String>,
    #[serde(default = "default_talk_durations")]
    pub talk_durations: Vec<u16>,
    #[serde(default = "default_duration")]
    pub default_duration: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub from_name: String,
    pub from_email: String,
    pub reply_to: String,
    #[serde(default)]
    pub templates: EmailTemplates,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EmailTemplates {
    #[serde(default)]
    pub submission_confirmation: Option<String>,
    #[serde(default)]
    pub acceptance_notification: Option<String>,
    #[serde(default)]
    pub rejection_notification: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LabelsConfig {
    #[serde(default = "default_labels")]
    pub default_labels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduleConfig {
    #[serde(default = "default_true")]
    pub enable_tracks: bool,
    #[serde(default = "default_track_names")]
    pub track_names: Vec<String>,
    #[serde(default = "default_slot_duration")]
    pub slot_duration_minutes: u16,
    #[serde(default = "default_break_duration")]
    pub break_duration_minutes: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry_hours: i64,
    #[serde(default = "default_session_timeout")]
    pub session_timeout_hours: i64,
    #[serde(default = "default_true")]
    pub enable_rate_limiting: bool,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_requests_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadsConfig {
    #[serde(default = "default_upload_dir")]
    pub directory: String,
    #[serde(default = "default_max_slide_size")]
    pub max_size_mb: u64,
    #[serde(default = "default_allowed_extensions")]
    pub allowed_extensions: Vec<String>,
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_min_title_length() -> usize {
    5
}

fn default_max_title_length() -> usize {
    200
}

fn default_min_summary_length() -> usize {
    50
}

fn default_max_summary_length() -> usize {
    500
}

fn default_max_description_length() -> usize {
    5000
}

fn default_max_slide_size() -> u64 {
    50
}

fn default_slide_formats() -> Vec<String> {
    vec![
        "pdf".to_string(),
        "ppt".to_string(),
        "pptx".to_string(),
        "odp".to_string(),
    ]
}

fn default_talk_durations() -> Vec<u16> {
    vec![20, 45, 60, 90]
}

fn default_duration() -> u16 {
    45
}

fn default_labels() -> Vec<String> {
    vec![
        "Linux".to_string(),
        "Open Source".to_string(),
        "DevOps".to_string(),
        "Security".to_string(),
        "Cloud".to_string(),
        "Containers".to_string(),
        "Networking".to_string(),
        "Embedded".to_string(),
        "Desktop".to_string(),
        "Server".to_string(),
        "Beginner".to_string(),
        "Intermediate".to_string(),
        "Advanced".to_string(),
        "Workshop".to_string(),
        "Tutorial".to_string(),
        "Case Study".to_string(),
    ]
}

fn default_track_names() -> Vec<String> {
    vec![
        "Main Hall".to_string(),
        "Workshop Room".to_string(),
        "Technical Track".to_string(),
    ]
}

fn default_slot_duration() -> u16 {
    45
}

fn default_break_duration() -> u16 {
    15
}

fn default_max_connections() -> u32 {
    10
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_jwt_expiry() -> i64 {
    24
}

fn default_session_timeout() -> i64 {
    168
}

fn default_rate_limit() -> u32 {
    60
}

fn default_upload_dir() -> String {
    "./uploads".to_string()
}

fn default_allowed_extensions() -> Vec<String> {
    vec![
        "pdf".to_string(),
        "ppt".to_string(),
        "pptx".to_string(),
        "odp".to_string(),
        "jpg".to_string(),
        "png".to_string(),
    ]
}
