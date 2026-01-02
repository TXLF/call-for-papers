use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use call_for_papers::api::create_router;
use call_for_papers::config::Config;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;

/// Test configuration
#[allow(dead_code)]
pub struct TestContext {
    pub db: PgPool,
    pub app: Router,
    pub config: Config,
    pub base_url: String,
}

#[allow(dead_code)]
impl TestContext {
    /// Create a new test context with isolated database
    pub async fn new() -> Self {
        // Load test config
        std::env::set_var("JWT_SECRET", "test_jwt_secret_for_integration_tests");
        std::env::set_var("DATABASE_URL", get_test_database_url());

        let config = Config::load().expect("Failed to load test config");

        // Create test database connection
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("Failed to run migrations");

        // Create router
        let app = create_router(db.clone(), config.clone());

        Self {
            db,
            app,
            config,
            base_url: "http://localhost".to_string(),
        }
    }

    /// Clean up the database after tests
    pub async fn cleanup(&self) {
        // Truncate all tables in reverse dependency order
        let tables = vec![
            "talk_labels",
            "ratings",
            "schedule_slots",
            "tracks",
            "conferences",
            "email_templates",
            "labels",
            "talks",
            "auth_providers",
            "sessions",
            "users",
        ];

        for table in tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.db)
                .await
                .ok();
        }
    }

    /// Make an HTTP request to the app
    pub async fn request(&self, req: Request<Body>) -> (StatusCode, Value) {
        let response = self
            .app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to send request");

        let status = response.status();
        let body = response.into_body();
        let bytes = body
            .collect()
            .await
            .expect("Failed to read body")
            .to_bytes();

        let json: Value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };

        (status, json)
    }

    /// Make a typed HTTP request
    pub async fn request_typed<T: DeserializeOwned>(
        &self,
        req: Request<Body>,
    ) -> (StatusCode, Option<T>) {
        let (status, value) = self.request(req).await;

        let typed = if value.is_null() {
            None
        } else {
            serde_json::from_value(value).ok()
        };

        (status, typed)
    }
}

/// Get test database URL (uses different database for tests)
fn get_test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/call_for_papers_test".to_string()
    })
}

/// Create a test user
#[allow(dead_code)]
pub async fn create_test_user(
    db: &PgPool,
    email: &str,
    username: &str,
    password: &str,
    full_name: &str,
    is_organizer: bool,
) -> uuid::Uuid {
    let password_hash =
        call_for_papers::handlers::auth::hash_password(password).expect("Failed to hash password");

    let user_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO users (email, username, password_hash, full_name, is_organizer)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(username)
    .bind(password_hash)
    .bind(full_name)
    .bind(is_organizer)
    .fetch_one(db)
    .await
    .expect("Failed to create test user");

    user_id
}

/// Generate JWT token for a user
pub fn generate_test_token(user_id: uuid::Uuid, email: &str, is_organizer: bool) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        email: String,
        is_organizer: bool,
        exp: u64,
        iat: u64,
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        is_organizer,
        exp: now + 3600, // 1 hour expiry
        iat: now,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test_jwt_secret".to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to generate token")
}

/// Create a test label
#[allow(dead_code)]
pub async fn create_test_label(db: &PgPool, name: &str, color: &str) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO labels (name, color)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(color)
    .fetch_one(db)
    .await
    .expect("Failed to create test label")
}

/// Create a test conference
#[allow(dead_code)]
pub async fn create_test_conference(db: &PgPool, name: &str) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO conferences (name, description, start_date, end_date, location, is_active)
        VALUES ($1, 'Test Conference', '2025-04-18', '2025-04-20', 'Test City', true)
        RETURNING id
        "#,
    )
    .bind(name)
    .fetch_one(db)
    .await
    .expect("Failed to create test conference")
}

/// Create a test track
#[allow(dead_code)]
pub async fn create_test_track(db: &PgPool, conference_id: uuid::Uuid, name: &str) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO tracks (conference_id, name, capacity)
        VALUES ($1, $2, 100)
        RETURNING id
        "#,
    )
    .bind(conference_id)
    .bind(name)
    .fetch_one(db)
    .await
    .expect("Failed to create test track")
}

/// Create a test talk
#[allow(dead_code)]
pub async fn create_test_talk(
    db: &PgPool,
    speaker_id: uuid::Uuid,
    title: &str,
    summary: &str,
) -> uuid::Uuid {
    sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO talks (speaker_id, title, short_summary, state)
        VALUES ($1, $2, $3, 'submitted')
        RETURNING id
        "#,
    )
    .bind(speaker_id)
    .bind(title)
    .bind(summary)
    .fetch_one(db)
    .await
    .expect("Failed to create test talk")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_context_creation() {
        let ctx = TestContext::new().await;
        assert!(!ctx.db.is_closed());
        ctx.cleanup().await;
    }

    #[tokio::test]
    #[serial]
    async fn test_create_test_user() {
        let ctx = TestContext::new().await;
        let user_id = create_test_user(
            &ctx.db,
            "test@example.com",
            "testuser",
            "password123",
            "Test User",
            false,
        )
        .await;
        assert!(user_id != uuid::Uuid::nil());
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_generate_test_token() {
        let user_id = uuid::Uuid::new_v4();
        let token = generate_test_token(user_id, "test@example.com", false);
        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ"));
    }
}
