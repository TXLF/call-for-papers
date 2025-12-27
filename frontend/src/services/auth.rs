use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};

use crate::types::{AuthResponse, ErrorResponse, LoginRequest, RegisterRequest};

const TOKEN_KEY: &str = "auth_token";

pub struct AuthService;

impl AuthService {
    pub async fn register(request: RegisterRequest) -> Result<AuthResponse, String> {
        let response = Request::post("/api/auth/register")
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let auth_response = response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            // Store token in local storage
            LocalStorage::set(TOKEN_KEY, &auth_response.token)
                .map_err(|e| format!("Failed to store token: {:?}", e))?;

            Ok(auth_response)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub async fn login(request: LoginRequest) -> Result<AuthResponse, String> {
        let response = Request::post("/api/auth/login")
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            let auth_response = response
                .json::<AuthResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            // Store token in local storage
            LocalStorage::set(TOKEN_KEY, &auth_response.token)
                .map_err(|e| format!("Failed to store token: {:?}", e))?;

            Ok(auth_response)
        } else {
            let error = response
                .json::<ErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error: {}", e))?;
            Err(error.error)
        }
    }

    pub fn logout() {
        LocalStorage::delete(TOKEN_KEY);
    }

    pub fn get_token() -> Option<String> {
        LocalStorage::get(TOKEN_KEY).ok()
    }

    pub fn is_authenticated() -> bool {
        Self::get_token().is_some()
    }
}
