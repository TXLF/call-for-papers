use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub upload_dir: String,
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
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/call_for_papers".to_string());

        let server_host = std::env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("SERVER_PORT must be a valid port number");

        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set in environment variables");

        let jwt_expiry_hours = std::env::var("JWT_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .expect("JWT_EXPIRY_HOURS must be a valid number");

        let upload_dir = std::env::var("UPLOAD_DIR")
            .unwrap_or_else(|_| "./uploads".to_string());

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

        Ok(Config {
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
        })
    }
}
