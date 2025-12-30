use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::{AuthProviderType, ErrorResponse, GoogleUserInfo, OAuthCallbackQuery},
        user::UserResponse,
        AuthResponse, Claims, LoginRequest, RegisterRequest, User
    },
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate email format
    if !payload.email.contains('@') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Invalid email format")),
        ));
    }

    // Validate password strength
    if payload.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Password must be at least 8 characters")),
        ));
    }

    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Internal server error")),
        )
    })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse::new("User with this email already exists")),
        ));
    }

    // Check if username is taken (if provided)
    if let Some(ref username) = payload.username {
        let existing_username = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Internal server error")),
            )
        })?;

        if existing_username.is_some() {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse::new("Username already taken")),
            ));
        }
    }

    // Hash password
    let password_hash = hash_password(&payload.password).map_err(|e| {
        tracing::error!("Password hashing error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Internal server error")),
        )
    })?;

    // Create user
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, username, password_hash, full_name, bio, is_organizer)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&payload.email)
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(&payload.bio)
    .bind(false) // New users are not organizers by default
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create user")),
        )
    })?;

    // Create session token
    let token = create_session_token(&state.db, &user, &state.config).await.map_err(|e| {
        tracing::error!("Token creation error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create session")),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Internal server error")),
            )
        })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid email or password")),
        )
    })?;

    // Verify password
    let password_hash = user.password_hash.as_ref().ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid email or password")),
        )
    })?;

    verify_password(&payload.password, password_hash).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid email or password")),
        )
    })?;

    // Create session token
    let token = create_session_token(&state.db, &user, &state.config).await.map_err(|e| {
        tracing::error!("Token creation error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to create session")),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}

async fn create_session_token(pool: &PgPool, user: &User, config: &crate::config::Config) -> Result<String, anyhow::Error> {
    // Generate JWT token
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        is_organizer: user.is_organizer,
        exp: (Utc::now() + chrono::Duration::hours(config.jwt_expiry_hours)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    // Store session in database
    let expires_at = Utc::now() + chrono::Duration::hours(config.jwt_expiry_hours);

    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user.id)
    .bind(&token)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn verify_token(token: &str, pool: &PgPool, jwt_secret: &str) -> Result<User, anyhow::Error> {
    // Verify JWT
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )?;

    // Check if session exists and is valid
    let session_check: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sessions
            WHERE token = $1 AND expires_at > NOW()
        )
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    let session_exists = session_check.map(|(exists,)| exists).unwrap_or(false);

    if !session_exists {
        return Err(anyhow::anyhow!("Invalid or expired session"));
    }

    // Get user
    let user_id = Uuid::parse_str(&token_data.claims.sub)?;
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

// Google OAuth handlers

fn get_google_oauth_client(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<BasicClient, anyhow::Error> {
    let google_client_id = ClientId::new(client_id.to_string());
    let google_client_secret = ClientSecret::new(client_secret.to_string());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?;

    Ok(BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?))
}

pub async fn google_authorize(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if Google OAuth is configured
    let client_id = state
        .config
        .google_client_id
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Google OAuth is not configured")),
            )
        })?;

    let client_secret = state
        .config
        .google_client_secret
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Google OAuth is not configured")),
            )
        })?;

    let redirect_url = state
        .config
        .google_redirect_url
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Google OAuth is not configured")),
            )
        })?;

    let client = get_google_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("OAuth configuration error")),
        )
    })?;

    // Generate the authorization URL
    let (authorize_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Ok(Redirect::to(authorize_url.as_str()))
}

pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if Google OAuth is configured
    let client_id = state
        .config
        .google_client_id
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Google OAuth is not configured")),
            )
        })?;

    let client_secret = state
        .config
        .google_client_secret
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Google OAuth is not configured")),
            )
        })?;

    let redirect_url = state
        .config
        .google_redirect_url
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Google OAuth is not configured")),
            )
        })?;

    let client = get_google_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("OAuth configuration error")),
        )
    })?;

    // Exchange the code for an access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            tracing::error!("Failed to exchange code for token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to authenticate with Google")),
            )
        })?;

    // Get user info from Google
    let user_info_url = "https://www.googleapis.com/oauth2/v3/userinfo";
    let client = reqwest::Client::new();
    let user_info_response = client
        .get(user_info_url)
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to fetch user info from Google")),
            )
        })?;

    let google_user: GoogleUserInfo = user_info_response.json().await.map_err(|e| {
        tracing::error!("Failed to parse user info: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to parse user info from Google")),
        )
    })?;

    // Check if user already exists with this Google account
    let existing_provider: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id FROM auth_providers
        WHERE provider = $1 AND provider_user_id = $2
        "#,
    )
    .bind(AuthProviderType::Google)
    .bind(&google_user.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Internal server error")),
        )
    })?;

    let user = if let Some((user_id,)) = existing_provider {
        // User exists, fetch the user
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Internal server error")),
                )
            })?
    } else {
        // Check if user exists with this email (from local registration)
        let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&google_user.email)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Internal server error")),
                )
            })?;

        if let Some(user) = existing_user {
            // Link Google account to existing user
            sqlx::query(
                r#"
                INSERT INTO auth_providers (user_id, provider, provider_user_id, provider_data)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(user.id)
            .bind(AuthProviderType::Google)
            .bind(&google_user.sub)
            .bind(serde_json::json!({
                "email": google_user.email,
                "name": google_user.name,
                "picture": google_user.picture,
            }))
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to link Google account")),
                )
            })?;

            user
        } else {
            // Create new user
            let new_user = sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (email, full_name, password_hash, is_organizer)
                VALUES ($1, $2, NULL, $3)
                RETURNING *
                "#,
            )
            .bind(&google_user.email)
            .bind(&google_user.name)
            .bind(false)
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to create user")),
                )
            })?;

            // Create auth provider record
            sqlx::query(
                r#"
                INSERT INTO auth_providers (user_id, provider, provider_user_id, provider_data)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(new_user.id)
            .bind(AuthProviderType::Google)
            .bind(&google_user.sub)
            .bind(serde_json::json!({
                "email": google_user.email,
                "name": google_user.name,
                "picture": google_user.picture,
            }))
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to create auth provider")),
                )
            })?;

            new_user
        }
    };

    // Create session token
    let token = create_session_token(&state.db, &user, &state.config)
        .await
        .map_err(|e| {
            tracing::error!("Token creation error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to create session")),
            )
        })?;

    // Redirect to frontend with token
    let redirect_url = format!("/auth/callback?token={}", token);
    Ok(Redirect::to(&redirect_url))
}
