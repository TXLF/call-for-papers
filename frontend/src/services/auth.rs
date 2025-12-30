use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::Deserialize;
use base64::{Engine as _, engine::general_purpose};

use crate::types::{AuthResponse, ErrorResponse, LoginRequest, RegisterRequest};

#[derive(Debug, Deserialize)]
struct JwtClaims {
    is_organizer: bool,
}

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

    pub fn set_token(token: &str) {
        let _ = LocalStorage::set(TOKEN_KEY, token);
    }

    pub fn is_authenticated() -> bool {
        Self::get_token().is_some()
    }

    /// Decode JWT token and extract claims
    fn decode_jwt_claims(token: &str) -> Option<JwtClaims> {
        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        // Decode the payload (second part)
        let payload = parts[1];

        // JWT uses base64url encoding, which may need padding
        let padded = match payload.len() % 4 {
            0 => payload.to_string(),
            n => format!("{}{}", payload, "=".repeat(4 - n)),
        };

        // Decode base64
        let decoded = general_purpose::STANDARD
            .decode(padded.as_bytes())
            .ok()?;

        // Parse JSON
        serde_json::from_slice::<JwtClaims>(&decoded).ok()
    }

    /// Check if the current user is an organizer by decoding the JWT token
    pub fn is_organizer() -> bool {
        Self::get_token()
            .and_then(|token| Self::decode_jwt_claims(&token))
            .map(|claims| claims.is_organizer)
            .unwrap_or(false)
    }
}
