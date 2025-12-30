mod models;

pub use models::*;

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileConfig {
    pub conference: ConferenceConfig,
    pub branding: BrandingConfig,
    pub features: FeaturesConfig,
    pub submission: SubmissionConfig,
    pub email: EmailConfig,
    pub labels: LabelsConfig,
    pub schedule: ScheduleConfig,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub uploads: UploadsConfig,
}

#[derive(Debug, Clone)]
pub struct Config {
    // File-based configuration
    pub conference: ConferenceConfig,
    pub branding: BrandingConfig,
    pub features: FeaturesConfig,
    pub submission: SubmissionConfig,
    pub email_config: EmailConfig,
    pub labels: LabelsConfig,
    pub schedule: ScheduleConfig,
    pub security: SecurityConfig,
    pub uploads: UploadsConfig,

    // Environment variable configuration (these override file config)
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub upload_dir: String,

    // OAuth configuration (from environment)
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_url: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub github_redirect_url: Option<String>,
    pub apple_client_id: Option<String>,
    pub apple_team_id: Option<String>,
    pub apple_key_id: Option<String>,
    pub apple_private_key: Option<String>,
    pub apple_redirect_url: Option<String>,
    pub facebook_client_id: Option<String>,
    pub facebook_client_secret: Option<String>,
    pub facebook_redirect_url: Option<String>,
    pub linkedin_client_id: Option<String>,
    pub linkedin_client_secret: Option<String>,
    pub linkedin_redirect_url: Option<String>,

    // SMTP configuration (from environment)
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_user: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: Option<String>,

    // AI API keys (from environment)
    pub claude_api_key: Option<String>,
    pub openai_api_key: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        // Load environment variables first
        dotenvy::dotenv().ok();

        // Try to load from config file, fallback to defaults if not found
        let file_config = Self::load_file_config("config.toml")
            .or_else(|_| Self::load_default_config())?;

        // Load environment variables (these take precedence)
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                if file_config.database.url.is_empty() {
                    "postgres://postgres:postgres@localhost/call_for_papers".to_string()
                } else {
                    file_config.database.url.clone()
                }
            });

        let server_host = std::env::var("SERVER_HOST")
            .unwrap_or_else(|_| file_config.server.host.clone());

        let server_port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(file_config.server.port);

        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in environment variables");

        let jwt_expiry_hours = std::env::var("JWT_EXPIRY_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(file_config.security.jwt_expiry_hours);

        let upload_dir = std::env::var("UPLOAD_DIR")
            .unwrap_or_else(|_| file_config.uploads.directory.clone());

        // OAuth configuration
        let google_client_id = std::env::var("GOOGLE_CLIENT_ID").ok();
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok();
        let google_redirect_url = std::env::var("GOOGLE_REDIRECT_URL").ok();

        let github_client_id = std::env::var("GITHUB_CLIENT_ID").ok();
        let github_client_secret = std::env::var("GITHUB_CLIENT_SECRET").ok();
        let github_redirect_url = std::env::var("GITHUB_REDIRECT_URL").ok();

        let apple_client_id = std::env::var("APPLE_CLIENT_ID").ok();
        let apple_team_id = std::env::var("APPLE_TEAM_ID").ok();
        let apple_key_id = std::env::var("APPLE_KEY_ID").ok();
        let apple_private_key = std::env::var("APPLE_PRIVATE_KEY").ok();
        let apple_redirect_url = std::env::var("APPLE_REDIRECT_URL").ok();

        let facebook_client_id = std::env::var("FACEBOOK_CLIENT_ID").ok();
        let facebook_client_secret = std::env::var("FACEBOOK_CLIENT_SECRET").ok();
        let facebook_redirect_url = std::env::var("FACEBOOK_REDIRECT_URL").ok();

        let linkedin_client_id = std::env::var("LINKEDIN_CLIENT_ID").ok();
        let linkedin_client_secret = std::env::var("LINKEDIN_CLIENT_SECRET").ok();
        let linkedin_redirect_url = std::env::var("LINKEDIN_REDIRECT_URL").ok();

        // SMTP configuration
        let smtp_host = std::env::var("SMTP_HOST").ok();
        let smtp_port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok());
        let smtp_user = std::env::var("SMTP_USER").ok();
        let smtp_password = std::env::var("SMTP_PASSWORD").ok();
        let smtp_from = std::env::var("SMTP_FROM").ok();

        // AI API keys
        let claude_api_key = std::env::var("CLAUDE_API_KEY").ok();
        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();

        Ok(Config {
            conference: file_config.conference,
            branding: file_config.branding,
            features: file_config.features,
            submission: file_config.submission,
            email_config: file_config.email,
            labels: file_config.labels,
            schedule: file_config.schedule,
            security: file_config.security,
            uploads: file_config.uploads,
            database_url,
            server_host,
            server_port,
            jwt_secret,
            jwt_expiry_hours,
            upload_dir,
            google_client_id,
            google_client_secret,
            google_redirect_url,
            github_client_id,
            github_client_secret,
            github_redirect_url,
            apple_client_id,
            apple_team_id,
            apple_key_id,
            apple_private_key,
            apple_redirect_url,
            facebook_client_id,
            facebook_client_secret,
            facebook_redirect_url,
            linkedin_client_id,
            linkedin_client_secret,
            linkedin_redirect_url,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
            smtp_from,
            claude_api_key,
            openai_api_key,
        })
    }

    fn load_file_config<P: AsRef<Path>>(path: P) -> Result<FileConfig, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: FileConfig = toml::from_str(&content)?;
        Ok(config)
    }

    fn load_default_config() -> Result<FileConfig, anyhow::Error> {
        // Return default TXLF configuration if no config file exists
        let default_toml = include_str!("../../config.default.toml");
        let config: FileConfig = toml::from_str(default_toml)?;
        Ok(config)
    }

    // Keep the old from_env method for backwards compatibility
    #[deprecated(note = "Use Config::load() instead")]
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Self::load()
    }
}
