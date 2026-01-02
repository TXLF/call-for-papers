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
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::AppState,
    models::{
        auth::{
            AppleUserData, AuthProviderType, ErrorResponse, FacebookUserInfo, GitHubEmail,
            GitHubUserInfo, GoogleUserInfo, LinkedInUserInfo, OAuthCallbackQuery,
        },
        user::UserResponse,
        AuthResponse, Claims, LoginRequest, RegisterRequest, User,
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
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
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
        let existing_username =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
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
    let token = create_session_token(&state.db, &user, &state.config)
        .await
        .map_err(|e| {
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
    let token = create_session_token(&state.db, &user, &state.config)
        .await
        .map_err(|e| {
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

fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}

async fn create_session_token(
    pool: &PgPool,
    user: &User,
    config: &crate::config::Config,
) -> Result<String, anyhow::Error> {
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

pub async fn verify_token(
    token: &str,
    pool: &PgPool,
    jwt_secret: &str,
) -> Result<User, anyhow::Error> {
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
    let client_id = state.config.google_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Google OAuth is not configured")),
        )
    })?;

    let client_secret = state.config.google_client_secret.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Google OAuth is not configured")),
        )
    })?;

    let redirect_url = state.config.google_redirect_url.as_ref().ok_or_else(|| {
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
    let client_id = state.config.google_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Google OAuth is not configured")),
        )
    })?;

    let client_secret = state.config.google_client_secret.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Google OAuth is not configured")),
        )
    })?;

    let redirect_url = state.config.google_redirect_url.as_ref().ok_or_else(|| {
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

// GitHub OAuth handlers

fn get_github_oauth_client(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<BasicClient, anyhow::Error> {
    let github_client_id = ClientId::new(client_id.to_string());
    let github_client_secret = ClientSecret::new(client_secret.to_string());
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?;
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?;

    Ok(BasicClient::new(
        github_client_id,
        Some(github_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?))
}

pub async fn github_authorize(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if GitHub OAuth is configured
    let client_id = state.config.github_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("GitHub OAuth is not configured")),
        )
    })?;

    let client_secret = state.config.github_client_secret.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("GitHub OAuth is not configured")),
        )
    })?;

    let redirect_url = state.config.github_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("GitHub OAuth is not configured")),
        )
    })?;

    let client = get_github_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("OAuth configuration error")),
        )
    })?;

    // Generate the authorization URL
    let (authorize_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    Ok(Redirect::to(authorize_url.as_str()))
}

pub async fn github_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if GitHub OAuth is configured
    let client_id = state.config.github_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("GitHub OAuth is not configured")),
        )
    })?;

    let client_secret = state.config.github_client_secret.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("GitHub OAuth is not configured")),
        )
    })?;

    let redirect_url = state.config.github_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("GitHub OAuth is not configured")),
        )
    })?;

    let client = get_github_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
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
                Json(ErrorResponse::new("Failed to authenticate with GitHub")),
            )
        })?;

    let http_client = reqwest::Client::new();

    // Get user info from GitHub
    let user_info_url = "https://api.github.com/user";
    let user_info_response = http_client
        .get(user_info_url)
        .bearer_auth(token_result.access_token().secret())
        .header("User-Agent", "Call-For-Papers")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to fetch user info from GitHub")),
            )
        })?;

    let github_user: GitHubUserInfo = user_info_response.json().await.map_err(|e| {
        tracing::error!("Failed to parse user info: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to parse user info from GitHub")),
        )
    })?;

    // Get user's email if not provided in user info
    let email = if let Some(email) = github_user.email {
        email
    } else {
        // Fetch emails from GitHub API
        let emails_url = "https://api.github.com/user/emails";
        let emails_response = http_client
            .get(emails_url)
            .bearer_auth(token_result.access_token().secret())
            .header("User-Agent", "Call-For-Papers")
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch user emails: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(
                        "Failed to fetch user emails from GitHub",
                    )),
                )
            })?;

        let github_emails: Vec<GitHubEmail> = emails_response.json().await.map_err(|e| {
            tracing::error!("Failed to parse user emails: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "Failed to parse user emails from GitHub",
                )),
            )
        })?;

        // Find primary verified email, or first verified email, or any email
        github_emails
            .iter()
            .find(|e| e.primary && e.verified)
            .or_else(|| github_emails.iter().find(|e| e.verified))
            .or_else(|| github_emails.first())
            .map(|e| e.email.clone())
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new("No email found in GitHub account")),
                )
            })?
    };

    // Check if user already exists with this GitHub account
    let github_user_id_str = github_user.id.to_string();
    let existing_provider: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id FROM auth_providers
        WHERE provider = $1 AND provider_user_id = $2
        "#,
    )
    .bind(AuthProviderType::Github)
    .bind(&github_user_id_str)
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
        // Check if user exists with this email (from local registration or other OAuth)
        let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&email)
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
            // Link GitHub account to existing user
            sqlx::query(
                r#"
                INSERT INTO auth_providers (user_id, provider, provider_user_id, provider_data)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(user.id)
            .bind(AuthProviderType::Github)
            .bind(&github_user_id_str)
            .bind(serde_json::json!({
                "id": github_user.id,
                "login": github_user.login,
                "email": email,
                "name": github_user.name,
                "avatar_url": github_user.avatar_url,
            }))
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to link GitHub account")),
                )
            })?;

            user
        } else {
            // Create new user
            let full_name = github_user
                .name
                .as_ref()
                .unwrap_or(&github_user.login)
                .clone();

            let new_user = sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (email, username, full_name, password_hash, is_organizer)
                VALUES ($1, $2, $3, NULL, $4)
                RETURNING *
                "#,
            )
            .bind(&email)
            .bind(&github_user.login)
            .bind(&full_name)
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
            .bind(AuthProviderType::Github)
            .bind(&github_user_id_str)
            .bind(serde_json::json!({
                "id": github_user.id,
                "login": github_user.login,
                "email": email,
                "name": github_user.name,
                "avatar_url": github_user.avatar_url,
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

// Apple OAuth handlers

fn get_apple_oauth_client(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<BasicClient, anyhow::Error> {
    let apple_client_id = ClientId::new(client_id.to_string());
    let apple_client_secret = ClientSecret::new(client_secret.to_string());
    let auth_url = AuthUrl::new("https://appleid.apple.com/auth/authorize".to_string())?;
    let token_url = TokenUrl::new("https://appleid.apple.com/auth/token".to_string())?;

    Ok(BasicClient::new(
        apple_client_id,
        Some(apple_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?))
}

pub async fn apple_authorize(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if Apple OAuth is configured
    let client_id = state.config.apple_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Apple OAuth is not configured")),
        )
    })?;

    let client_secret = state.config.apple_private_key.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Apple OAuth is not configured")),
        )
    })?;

    let redirect_url = state.config.apple_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Apple OAuth is not configured")),
        )
    })?;

    let client = get_apple_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("OAuth configuration error")),
        )
    })?;

    // Generate the authorization URL
    let (authorize_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("name".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(Redirect::to(authorize_url.as_str()))
}

pub async fn apple_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if Apple OAuth is configured
    let client_id = state.config.apple_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Apple OAuth is not configured")),
        )
    })?;

    let client_secret = state.config.apple_private_key.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Apple OAuth is not configured")),
        )
    })?;

    let redirect_url = state.config.apple_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Apple OAuth is not configured")),
        )
    })?;

    let client = get_apple_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
        tracing::error!("Failed to create OAuth client: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("OAuth configuration error")),
        )
    })?;

    // Exchange the code for an access token
    let _token_result = client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            tracing::error!("Failed to exchange code for token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to authenticate with Apple")),
            )
        })?;

    // Parse user data if provided (first-time login)
    let user_data: Option<AppleUserData> = query
        .user
        .as_ref()
        .and_then(|u| serde_json::from_str(u).ok());

    // Get email from user data - Apple only sends this on first authorization
    let email = user_data
        .as_ref()
        .and_then(|d| d.email.clone())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "Email required for first-time Apple sign-in. Please try again.",
                )),
            )
        })?;

    // Generate a unique identifier for this Apple user
    // Since we don't have access to the sub claim easily, use email as the identifier
    let apple_user_id = email.clone();

    // Check if user already exists with this Apple account
    let existing_provider: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id FROM auth_providers
        WHERE provider = $1 AND provider_user_id = $2
        "#,
    )
    .bind(AuthProviderType::Apple)
    .bind(&apple_user_id)
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
        // Check if user exists with this email (from local registration or other OAuth)
        let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&email)
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
            // Link Apple account to existing user
            sqlx::query(
                r#"
                INSERT INTO auth_providers (user_id, provider, provider_user_id, provider_data)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(user.id)
            .bind(AuthProviderType::Apple)
            .bind(&apple_user_id)
            .bind(serde_json::json!({
                "email": email,
            }))
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to link Apple account")),
                )
            })?;

            user
        } else {
            // Create new user
            let full_name = user_data
                .as_ref()
                .and_then(|d| {
                    d.name.as_ref().map(|n| {
                        format!(
                            "{} {}",
                            n.first_name.as_deref().unwrap_or(""),
                            n.last_name.as_deref().unwrap_or("")
                        )
                        .trim()
                        .to_string()
                    })
                })
                .unwrap_or_else(|| email.split('@').next().unwrap_or("User").to_string());

            let new_user = sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (email, full_name, password_hash, is_organizer)
                VALUES ($1, $2, NULL, $3)
                RETURNING *
                "#,
            )
            .bind(&email)
            .bind(&full_name)
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
            .bind(AuthProviderType::Apple)
            .bind(&apple_user_id)
            .bind(serde_json::json!({
                "email": email,
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

// Facebook OAuth handlers

fn get_facebook_oauth_client(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<BasicClient, anyhow::Error> {
    let facebook_client_id = ClientId::new(client_id.to_string());
    let facebook_client_secret = ClientSecret::new(client_secret.to_string());
    let auth_url = AuthUrl::new("https://www.facebook.com/v18.0/dialog/oauth".to_string())?;
    let token_url =
        TokenUrl::new("https://graph.facebook.com/v18.0/oauth/access_token".to_string())?;

    Ok(BasicClient::new(
        facebook_client_id,
        Some(facebook_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?))
}

pub async fn facebook_authorize(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if Facebook OAuth is configured
    let client_id = state.config.facebook_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Facebook OAuth is not configured")),
        )
    })?;

    let client_secret = state
        .config
        .facebook_client_secret
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Facebook OAuth is not configured")),
            )
        })?;

    let redirect_url = state.config.facebook_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Facebook OAuth is not configured")),
        )
    })?;

    let client =
        get_facebook_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
            tracing::error!("Failed to create OAuth client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("OAuth configuration error")),
            )
        })?;

    // Generate the authorization URL
    let (authorize_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("public_profile".to_string()))
        .url();

    Ok(Redirect::to(authorize_url.as_str()))
}

pub async fn facebook_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if Facebook OAuth is configured
    let client_id = state.config.facebook_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Facebook OAuth is not configured")),
        )
    })?;

    let client_secret = state
        .config
        .facebook_client_secret
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse::new("Facebook OAuth is not configured")),
            )
        })?;

    let redirect_url = state.config.facebook_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("Facebook OAuth is not configured")),
        )
    })?;

    let client =
        get_facebook_oauth_client(client_id, client_secret, redirect_url).map_err(|e| {
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
                Json(ErrorResponse::new("Failed to authenticate with Facebook")),
            )
        })?;

    let http_client = reqwest::Client::new();

    // Get user info from Facebook Graph API
    let user_info_url = "https://graph.facebook.com/v18.0/me?fields=id,name,email";
    let user_info_response = http_client
        .get(user_info_url)
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user info: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "Failed to fetch user info from Facebook",
                )),
            )
        })?;

    let facebook_user: FacebookUserInfo = user_info_response.json().await.map_err(|e| {
        tracing::error!("Failed to parse user info: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(
                "Failed to parse user info from Facebook",
            )),
        )
    })?;

    // Email is required
    let email = facebook_user.email.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "Email permission required for Facebook login",
            )),
        )
    })?;

    // Check if user already exists with this Facebook account
    let existing_provider: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id FROM auth_providers
        WHERE provider = $1 AND provider_user_id = $2
        "#,
    )
    .bind(AuthProviderType::Facebook)
    .bind(&facebook_user.id)
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
        // Check if user exists with this email (from local registration or other OAuth)
        let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&email)
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
            // Link Facebook account to existing user
            sqlx::query(
                r#"
                INSERT INTO auth_providers (user_id, provider, provider_user_id, provider_data)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(user.id)
            .bind(AuthProviderType::Facebook)
            .bind(&facebook_user.id)
            .bind(serde_json::json!({
                "id": facebook_user.id,
                "name": facebook_user.name,
                "email": email,
            }))
            .execute(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to link Facebook account")),
                )
            })?;

            user
        } else {
            // Create new user
            let full_name = facebook_user
                .name
                .as_ref()
                .unwrap_or(&email.split('@').next().unwrap_or("User").to_string())
                .clone();

            let new_user = sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (email, full_name, password_hash, is_organizer)
                VALUES ($1, $2, NULL, $3)
                RETURNING *
                "#,
            )
            .bind(&email)
            .bind(&full_name)
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
            .bind(AuthProviderType::Facebook)
            .bind(&facebook_user.id)
            .bind(serde_json::json!({
                "id": facebook_user.id,
                "name": facebook_user.name,
                "email": email,
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

// LinkedIn OAuth helpers
fn get_linkedin_oauth_client(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<BasicClient, anyhow::Error> {
    let auth_url = AuthUrl::new("https://www.linkedin.com/oauth/v2/authorization".to_string())?;
    let token_url = TokenUrl::new("https://www.linkedin.com/oauth/v2/accessToken".to_string())?;

    let client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);

    Ok(client)
}

pub async fn linkedin_authorize(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let linkedin_client_id = state.config.linkedin_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("LinkedIn OAuth not configured")),
        )
    })?;

    let linkedin_client_secret = state
        .config
        .linkedin_client_secret
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("LinkedIn OAuth not configured")),
            )
        })?;

    let linkedin_redirect_url = state.config.linkedin_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("LinkedIn OAuth not configured")),
        )
    })?;

    let client = get_linkedin_oauth_client(
        linkedin_client_id,
        linkedin_client_secret,
        linkedin_redirect_url,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Failed to create LinkedIn OAuth client: {}",
                e
            ))),
        )
    })?;

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(Redirect::to(auth_url.as_str()))
}

pub async fn linkedin_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let linkedin_client_id = state.config.linkedin_client_id.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("LinkedIn OAuth not configured")),
        )
    })?;

    let linkedin_client_secret = state
        .config
        .linkedin_client_secret
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("LinkedIn OAuth not configured")),
            )
        })?;

    let linkedin_redirect_url = state.config.linkedin_redirect_url.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("LinkedIn OAuth not configured")),
        )
    })?;

    let client = get_linkedin_oauth_client(
        linkedin_client_id,
        linkedin_client_secret,
        linkedin_redirect_url,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Failed to create LinkedIn OAuth client: {}",
                e
            ))),
        )
    })?;

    // Exchange the code for an access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to exchange code for token: {}",
                    e
                ))),
            )
        })?;

    let access_token = token_result.access_token().secret();

    // Fetch user info from LinkedIn using OpenID Connect userinfo endpoint
    let client = reqwest::Client::new();
    let linkedin_user: LinkedInUserInfo = client
        .get("https://api.linkedin.com/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to fetch LinkedIn user info: {}",
                    e
                ))),
            )
        })?
        .json()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to parse LinkedIn user info: {}",
                    e
                ))),
            )
        })?;

    // Check if email is provided (it's required by our scope)
    let email = linkedin_user.email.clone();

    // Check if this LinkedIn account is already linked
    let existing_provider = sqlx::query!(
        r#"
        SELECT user_id
        FROM auth_providers
        WHERE provider = 'linkedin' AND provider_user_id = $1
        "#,
        linkedin_user.sub
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    let user_id = if let Some(provider) = existing_provider {
        // User already exists with this LinkedIn account
        provider.user_id
    } else {
        // Check if a user with this email already exists
        let existing_user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, username, password_hash, full_name, bio, is_organizer, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Database error: {}", e))),
            )
        })?;

        let user_id = if let Some(user) = existing_user {
            // Link this LinkedIn account to the existing user
            user.id
        } else {
            // Create a new user
            let new_user = sqlx::query!(
                r#"
                INSERT INTO users (email, full_name, password_hash, is_organizer)
                VALUES ($1, $2, NULL, FALSE)
                RETURNING id
                "#,
                email,
                linkedin_user.name
            )
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Failed to create user: {}", e))),
                )
            })?;

            new_user.id
        };

        // Link the LinkedIn account to the user
        sqlx::query!(
            r#"
            INSERT INTO auth_providers (user_id, provider, provider_user_id, provider_data)
            VALUES ($1, 'linkedin', $2, $3)
            "#,
            user_id,
            linkedin_user.sub,
            serde_json::to_value(&linkedin_user).unwrap()
        )
        .execute(&state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to link LinkedIn account: {}",
                    e
                ))),
            )
        })?;

        user_id
    };

    // Fetch the complete user
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, username, password_hash, full_name, bio, is_organizer, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Database error: {}", e))),
        )
    })?;

    // Generate JWT token
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        is_organizer: user.is_organizer,
        exp: (Utc::now() + chrono::Duration::hours(state.config.jwt_expiry_hours)).timestamp()
            as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Failed to generate token: {}",
                e
            ))),
        )
    })?;

    // Store session
    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user.id,
        token,
        Utc::now() + chrono::Duration::hours(state.config.jwt_expiry_hours)
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Failed to create session: {}",
                e
            ))),
        )
    })?;

    // Redirect to frontend with token
    Ok(Redirect::to(&format!("/auth/callback?token={}", token)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_creates_valid_hash() {
        let password = "secure_password_123";
        let result = hash_password(password);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_generates_different_hashes() {
        let password = "test_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should generate different hashes due to different salts
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_password_success() {
        let password = "my_secure_password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_password_wrong_password() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        let result = verify_password("wrong_password", &hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("password", "invalid_hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new("Test error message");
        assert_eq!(error.error, "Test error message");
    }

    #[test]
    fn test_error_response_from_string() {
        let error = ErrorResponse::new(String::from("Another error"));
        assert_eq!(error.error, "Another error");
    }

    #[test]
    fn test_auth_provider_type_equality() {
        assert_eq!(AuthProviderType::Google, AuthProviderType::Google);
        assert_ne!(AuthProviderType::Google, AuthProviderType::Github);
        assert_ne!(AuthProviderType::Local, AuthProviderType::Apple);
    }
}
