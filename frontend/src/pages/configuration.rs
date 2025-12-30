use yew::prelude::*;
use gloo_net::http::Request;
use serde::Deserialize;
use crate::services::auth::AuthService;

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct ConfigResponse {
    conference: ConferenceInfo,
    branding: BrandingInfo,
    features: FeaturesInfo,
    submission: SubmissionInfo,
    email: EmailInfo,
    labels: LabelsInfo,
    schedule: ScheduleInfo,
    security: SecurityInfo,
    uploads: UploadsInfo,
    integrations: IntegrationsInfo,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct ConferenceInfo {
    name: String,
    short_name: String,
    description: String,
    year: u16,
    website: String,
    submission_open: String,
    submission_close: String,
    conference_start: String,
    conference_end: String,
    location: String,
    venue: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct BrandingInfo {
    primary_color: String,
    secondary_color: String,
    accent_color: String,
    background_color: String,
    text_color: String,
    logo_light: String,
    logo_dark: String,
    favicon: String,
    custom_css: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct FeaturesInfo {
    enable_speaker_registration: bool,
    enable_organizer_registration: bool,
    enable_social_login: bool,
    enable_google_login: bool,
    enable_github_login: bool,
    enable_apple_login: bool,
    enable_facebook_login: bool,
    enable_linkedin_login: bool,
    enable_slide_upload: bool,
    enable_ratings: bool,
    enable_ai_tagging: bool,
    enable_schedule_builder: bool,
    require_speaker_confirmation: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct SubmissionInfo {
    min_title_length: usize,
    max_title_length: usize,
    min_summary_length: usize,
    max_summary_length: usize,
    max_description_length: usize,
    max_slide_size_mb: u64,
    allowed_slide_formats: Vec<String>,
    talk_durations: Vec<u16>,
    default_duration: u16,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct EmailInfo {
    from_name: String,
    from_email: String,
    reply_to: String,
    smtp_configured: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct LabelsInfo {
    default_labels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct ScheduleInfo {
    enable_tracks: bool,
    track_names: Vec<String>,
    slot_duration_minutes: u16,
    break_duration_minutes: u16,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct SecurityInfo {
    jwt_expiry_hours: i64,
    session_timeout_hours: i64,
    enable_rate_limiting: bool,
    rate_limit_requests_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct UploadsInfo {
    directory: String,
    max_size_mb: u64,
    allowed_extensions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct IntegrationsInfo {
    google_oauth_configured: bool,
    github_oauth_configured: bool,
    apple_oauth_configured: bool,
    facebook_oauth_configured: bool,
    linkedin_oauth_configured: bool,
    claude_api_configured: bool,
    openai_api_configured: bool,
    smtp_configured: bool,
}

#[function_component(Configuration)]
pub fn configuration() -> Html {
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let config = use_state(|| None::<ConfigResponse>);

    {
        let loading = loading.clone();
        let error = error.clone();
        let config = config.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);

                let token = match AuthService::get_token() {
                    Some(t) => t,
                    None => {
                        error.set(Some("Not authenticated".to_string()));
                        loading.set(false);
                        return;
                    }
                };

                match Request::get("/api/configuration")
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<ConfigResponse>().await {
                                Ok(data) => {
                                    config.set(Some(data));
                                }
                                Err(e) => {
                                    error.set(Some(format!("Failed to parse response: {}", e)));
                                }
                            }
                        } else {
                            match response.text().await {
                                Ok(text) => error.set(Some(format!("Failed to load configuration: {}", text))),
                                Err(e) => error.set(Some(format!("Failed to load configuration: {}", e))),
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Request failed: {}", e)));
                    }
                }

                loading.set(false);
            });

            || ()
        });
    }

    html! {
        <div class="configuration-container">
            <div class="page-header">
                <h1>{ "System Configuration" }</h1>
                <p class="subtitle">{ "View current system configuration and settings" }</p>
            </div>

            if *loading {
                <div class="loading">{ "Loading configuration..." }</div>
            } else if let Some(err) = (*error).as_ref() {
                <div class="error-message">{ err }</div>
            } else if let Some(cfg) = (*config).as_ref() {
                <>
                    <div class="config-notice">
                        <strong>{ "Note:" }</strong>
                        { " Configuration is currently read-only. To modify settings, update your " }
                        <code>{ "config.toml" }</code>
                        { " file and restart the server." }
                    </div>

                    // Conference Section
                    <div class="config-section">
                        <h2>{ "Conference Information" }</h2>
                        <div class="config-grid">
                            <div class="config-item">
                                <label>{ "Name:" }</label>
                                <span>{ &cfg.conference.name }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Short Name:" }</label>
                                <span>{ &cfg.conference.short_name }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Year:" }</label>
                                <span>{ cfg.conference.year }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Website:" }</label>
                                <span><a href={cfg.conference.website.clone()} target="_blank">{ &cfg.conference.website }</a></span>
                            </div>
                            <div class="config-item">
                                <label>{ "Location:" }</label>
                                <span>{ &cfg.conference.location }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Venue:" }</label>
                                <span>{ &cfg.conference.venue }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Submissions Open:" }</label>
                                <span>{ &cfg.conference.submission_open }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Submissions Close:" }</label>
                                <span>{ &cfg.conference.submission_close }</span>
                            </div>
                            <div class="config-item full-width">
                                <label>{ "Description:" }</label>
                                <span>{ &cfg.conference.description }</span>
                            </div>
                        </div>
                    </div>

                    // Features Section
                    <div class="config-section">
                        <h2>{ "Features" }</h2>
                        <div class="config-grid">
                            <div class="config-item">
                                <label>{ "Speaker Registration:" }</label>
                                <span class={if cfg.features.enable_speaker_registration { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.enable_speaker_registration { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Organizer Registration:" }</label>
                                <span class={if cfg.features.enable_organizer_registration { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.enable_organizer_registration { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Social Login:" }</label>
                                <span class={if cfg.features.enable_social_login { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.enable_social_login { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Slide Upload:" }</label>
                                <span class={if cfg.features.enable_slide_upload { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.enable_slide_upload { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Talk Ratings:" }</label>
                                <span class={if cfg.features.enable_ratings { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.enable_ratings { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "AI Auto-Tagging:" }</label>
                                <span class={if cfg.features.enable_ai_tagging { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.enable_ai_tagging { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Schedule Builder:" }</label>
                                <span class={if cfg.features.enable_schedule_builder { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.enable_schedule_builder { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Require Speaker Confirmation:" }</label>
                                <span class={if cfg.features.require_speaker_confirmation { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.features.require_speaker_confirmation { "Yes" } else { "No" } }
                                </span>
                            </div>
                        </div>
                    </div>

                    // Branding Section
                    <div class="config-section">
                        <h2>{ "Branding" }</h2>
                        <div class="config-grid">
                            <div class="config-item">
                                <label>{ "Primary Color:" }</label>
                                <span>
                                    <span class="color-preview" style={format!("background-color: {}", cfg.branding.primary_color)}></span>
                                    { &cfg.branding.primary_color }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Secondary Color:" }</label>
                                <span>
                                    <span class="color-preview" style={format!("background-color: {}", cfg.branding.secondary_color)}></span>
                                    { &cfg.branding.secondary_color }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Accent Color:" }</label>
                                <span>
                                    <span class="color-preview" style={format!("background-color: {}", cfg.branding.accent_color)}></span>
                                    { &cfg.branding.accent_color }
                                </span>
                            </div>
                        </div>
                    </div>

                    // Submission Settings
                    <div class="config-section">
                        <h2>{ "Submission Settings" }</h2>
                        <div class="config-grid">
                            <div class="config-item">
                                <label>{ "Title Length:" }</label>
                                <span>{ format!("{}-{} characters", cfg.submission.min_title_length, cfg.submission.max_title_length) }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Summary Length:" }</label>
                                <span>{ format!("{}-{} characters", cfg.submission.min_summary_length, cfg.submission.max_summary_length) }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Max Description:" }</label>
                                <span>{ format!("{} characters", cfg.submission.max_description_length) }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Max Slide Size:" }</label>
                                <span>{ format!("{} MB", cfg.submission.max_slide_size_mb) }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Talk Durations:" }</label>
                                <span>{ format!("{:?} minutes", cfg.submission.talk_durations) }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Allowed Formats:" }</label>
                                <span>{ cfg.submission.allowed_slide_formats.join(", ") }</span>
                            </div>
                        </div>
                    </div>

                    // Integrations Section
                    <div class="config-section">
                        <h2>{ "Integrations" }</h2>
                        <div class="config-grid">
                            <div class="config-item">
                                <label>{ "Google OAuth:" }</label>
                                <span class={if cfg.integrations.google_oauth_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.google_oauth_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "GitHub OAuth:" }</label>
                                <span class={if cfg.integrations.github_oauth_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.github_oauth_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Apple OAuth:" }</label>
                                <span class={if cfg.integrations.apple_oauth_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.apple_oauth_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Facebook OAuth:" }</label>
                                <span class={if cfg.integrations.facebook_oauth_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.facebook_oauth_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "LinkedIn OAuth:" }</label>
                                <span class={if cfg.integrations.linkedin_oauth_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.linkedin_oauth_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "Claude API:" }</label>
                                <span class={if cfg.integrations.claude_api_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.claude_api_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "OpenAI API:" }</label>
                                <span class={if cfg.integrations.openai_api_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.openai_api_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                            <div class="config-item">
                                <label>{ "SMTP Email:" }</label>
                                <span class={if cfg.integrations.smtp_configured { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.integrations.smtp_configured { "Configured" } else { "Not Configured" } }
                                </span>
                            </div>
                        </div>
                    </div>

                    // Security Section
                    <div class="config-section">
                        <h2>{ "Security" }</h2>
                        <div class="config-grid">
                            <div class="config-item">
                                <label>{ "JWT Expiry:" }</label>
                                <span>{ format!("{} hours", cfg.security.jwt_expiry_hours) }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Session Timeout:" }</label>
                                <span>{ format!("{} hours", cfg.security.session_timeout_hours) }</span>
                            </div>
                            <div class="config-item">
                                <label>{ "Rate Limiting:" }</label>
                                <span class={if cfg.security.enable_rate_limiting { "status-enabled" } else { "status-disabled" }}>
                                    { if cfg.security.enable_rate_limiting { "Enabled" } else { "Disabled" } }
                                </span>
                            </div>
                            if cfg.security.enable_rate_limiting {
                                <div class="config-item">
                                    <label>{ "Rate Limit:" }</label>
                                    <span>{ format!("{} requests/minute", cfg.security.rate_limit_requests_per_minute) }</span>
                                </div>
                            }
                        </div>
                    </div>
                </>
            }
        </div>
    }
}
