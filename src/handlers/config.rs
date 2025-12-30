use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::api::AppState;

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub conference: ConferenceInfo,
    pub branding: BrandingInfo,
    pub features: FeaturesInfo,
    pub submission: SubmissionInfo,
    pub email: EmailInfo,
    pub labels: LabelsInfo,
    pub schedule: ScheduleInfo,
    pub security: SecurityInfo,
    pub uploads: UploadsInfo,
    pub integrations: IntegrationsInfo,
}

#[derive(Debug, Serialize)]
pub struct ConferenceInfo {
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

#[derive(Debug, Serialize)]
pub struct BrandingInfo {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub logo_light: String,
    pub logo_dark: String,
    pub favicon: String,
    pub custom_css: String,
}

#[derive(Debug, Serialize)]
pub struct FeaturesInfo {
    pub enable_speaker_registration: bool,
    pub enable_organizer_registration: bool,
    pub enable_social_login: bool,
    pub enable_google_login: bool,
    pub enable_github_login: bool,
    pub enable_apple_login: bool,
    pub enable_facebook_login: bool,
    pub enable_linkedin_login: bool,
    pub enable_slide_upload: bool,
    pub enable_ratings: bool,
    pub enable_ai_tagging: bool,
    pub enable_schedule_builder: bool,
    pub require_speaker_confirmation: bool,
}

#[derive(Debug, Serialize)]
pub struct SubmissionInfo {
    pub min_title_length: usize,
    pub max_title_length: usize,
    pub min_summary_length: usize,
    pub max_summary_length: usize,
    pub max_description_length: usize,
    pub max_slide_size_mb: u64,
    pub allowed_slide_formats: Vec<String>,
    pub talk_durations: Vec<u16>,
    pub default_duration: u16,
}

#[derive(Debug, Serialize)]
pub struct EmailInfo {
    pub from_name: String,
    pub from_email: String,
    pub reply_to: String,
    pub smtp_configured: bool,
}

#[derive(Debug, Serialize)]
pub struct LabelsInfo {
    pub default_labels: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleInfo {
    pub enable_tracks: bool,
    pub track_names: Vec<String>,
    pub slot_duration_minutes: u16,
    pub break_duration_minutes: u16,
}

#[derive(Debug, Serialize)]
pub struct SecurityInfo {
    pub jwt_expiry_hours: i64,
    pub session_timeout_hours: i64,
    pub enable_rate_limiting: bool,
    pub rate_limit_requests_per_minute: u32,
}

#[derive(Debug, Serialize)]
pub struct UploadsInfo {
    pub directory: String,
    pub max_size_mb: u64,
    pub allowed_extensions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IntegrationsInfo {
    pub google_oauth_configured: bool,
    pub github_oauth_configured: bool,
    pub apple_oauth_configured: bool,
    pub facebook_oauth_configured: bool,
    pub linkedin_oauth_configured: bool,
    pub claude_api_configured: bool,
    pub openai_api_configured: bool,
    pub smtp_configured: bool,
}

pub async fn get_configuration(
    State(state): State<AppState>,
) -> Result<Json<ConfigResponse>, (StatusCode, String)> {
    let config = &state.config;

    let response = ConfigResponse {
        conference: ConferenceInfo {
            name: config.conference.name.clone(),
            short_name: config.conference.short_name.clone(),
            description: config.conference.description.clone(),
            year: config.conference.year,
            website: config.conference.website.clone(),
            submission_open: config.conference.submission_open.clone(),
            submission_close: config.conference.submission_close.clone(),
            conference_start: config.conference.conference_start.clone(),
            conference_end: config.conference.conference_end.clone(),
            location: config.conference.location.clone(),
            venue: config.conference.venue.clone(),
        },
        branding: BrandingInfo {
            primary_color: config.branding.primary_color.clone(),
            secondary_color: config.branding.secondary_color.clone(),
            accent_color: config.branding.accent_color.clone(),
            background_color: config.branding.background_color.clone(),
            text_color: config.branding.text_color.clone(),
            logo_light: config.branding.logo_light.clone(),
            logo_dark: config.branding.logo_dark.clone(),
            favicon: config.branding.favicon.clone(),
            custom_css: config.branding.custom_css.clone(),
        },
        features: FeaturesInfo {
            enable_speaker_registration: config.features.enable_speaker_registration,
            enable_organizer_registration: config.features.enable_organizer_registration,
            enable_social_login: config.features.enable_social_login,
            enable_google_login: config.features.enable_google_login,
            enable_github_login: config.features.enable_github_login,
            enable_apple_login: config.features.enable_apple_login,
            enable_facebook_login: config.features.enable_facebook_login,
            enable_linkedin_login: config.features.enable_linkedin_login,
            enable_slide_upload: config.features.enable_slide_upload,
            enable_ratings: config.features.enable_ratings,
            enable_ai_tagging: config.features.enable_ai_tagging,
            enable_schedule_builder: config.features.enable_schedule_builder,
            require_speaker_confirmation: config.features.require_speaker_confirmation,
        },
        submission: SubmissionInfo {
            min_title_length: config.submission.min_title_length,
            max_title_length: config.submission.max_title_length,
            min_summary_length: config.submission.min_summary_length,
            max_summary_length: config.submission.max_summary_length,
            max_description_length: config.submission.max_description_length,
            max_slide_size_mb: config.submission.max_slide_size_mb,
            allowed_slide_formats: config.submission.allowed_slide_formats.clone(),
            talk_durations: config.submission.talk_durations.clone(),
            default_duration: config.submission.default_duration,
        },
        email: EmailInfo {
            from_name: config.email_config.from_name.clone(),
            from_email: config.email_config.from_email.clone(),
            reply_to: config.email_config.reply_to.clone(),
            smtp_configured: config.smtp_host.is_some() && config.smtp_port.is_some(),
        },
        labels: LabelsInfo {
            default_labels: config.labels.default_labels.clone(),
        },
        schedule: ScheduleInfo {
            enable_tracks: config.schedule.enable_tracks,
            track_names: config.schedule.track_names.clone(),
            slot_duration_minutes: config.schedule.slot_duration_minutes,
            break_duration_minutes: config.schedule.break_duration_minutes,
        },
        security: SecurityInfo {
            jwt_expiry_hours: config.security.jwt_expiry_hours,
            session_timeout_hours: config.security.session_timeout_hours,
            enable_rate_limiting: config.security.enable_rate_limiting,
            rate_limit_requests_per_minute: config.security.rate_limit_requests_per_minute,
        },
        uploads: UploadsInfo {
            directory: config.uploads.directory.clone(),
            max_size_mb: config.uploads.max_size_mb,
            allowed_extensions: config.uploads.allowed_extensions.clone(),
        },
        integrations: IntegrationsInfo {
            google_oauth_configured: config.google_client_id.is_some()
                && config.google_client_secret.is_some(),
            github_oauth_configured: config.github_client_id.is_some()
                && config.github_client_secret.is_some(),
            apple_oauth_configured: config.apple_client_id.is_some()
                && config.apple_team_id.is_some()
                && config.apple_key_id.is_some(),
            facebook_oauth_configured: config.facebook_client_id.is_some()
                && config.facebook_client_secret.is_some(),
            linkedin_oauth_configured: config.linkedin_client_id.is_some()
                && config.linkedin_client_secret.is_some(),
            claude_api_configured: config.claude_api_key.is_some(),
            openai_api_configured: config.openai_api_key.is_some(),
            smtp_configured: config.smtp_host.is_some() && config.smtp_port.is_some(),
        },
    };

    Ok(Json(response))
}
