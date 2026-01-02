use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "auth_provider_type", rename_all = "lowercase")]
pub enum AuthProviderType {
    Local,
    Google,
    Facebook,
    Github,
    Apple,
    Linkedin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthProvider {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: AuthProviderType,
    pub provider_user_id: String,
    pub provider_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String, // Google user ID
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUserInfo {
    pub id: i64,               // GitHub user ID
    pub login: String,         // GitHub username
    pub email: Option<String>, // May be null if private
    pub name: Option<String>,  // Display name
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppleUserInfo {
    pub sub: String, // Apple user ID
    pub email: Option<String>,
    pub email_verified: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppleTokenResponse {
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppleUserData {
    pub name: Option<AppleUserName>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppleUserName {
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FacebookUserInfo {
    pub id: String, // Facebook user ID
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LinkedInUserInfo {
    pub sub: String, // LinkedIn user ID
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
    pub user: Option<String>, // Apple sends user data as JSON string on first auth
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: Option<String>,
    pub password: String,
    pub full_name: String,
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: super::user::UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub email: String,
    pub is_organizer: bool,
    pub exp: usize, // Expiration time
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}
