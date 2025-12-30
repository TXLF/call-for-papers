mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::json;

#[tokio::test]
async fn test_register_success() {
    let ctx = TestContext::new().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "newuser@example.com",
                "username": "newuser",
                "password": "SecurePass123!",
                "full_name": "New User"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["token"].is_string());
    assert_eq!(response["user"]["email"], "newuser@example.com");
    assert_eq!(response["user"]["username"], "newuser");
    assert_eq!(response["user"]["full_name"], "New User");
    assert_eq!(response["user"]["is_organizer"], false);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let ctx = TestContext::new().await;

    // Create first user
    create_test_user(
        &ctx.db,
        "existing@example.com",
        "existinguser",
        "password",
        "Existing User",
        false,
    )
    .await;

    // Try to register with same email
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "existing@example.com",
                "username": "differentuser",
                "password": "SecurePass123!",
                "full_name": "Different User"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].is_string());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_register_invalid_email() {
    let ctx = TestContext::new().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "not-an-email",
                "username": "testuser",
                "password": "SecurePass123!",
                "full_name": "Test User"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_login_success() {
    let ctx = TestContext::new().await;

    // Create user
    create_test_user(
        &ctx.db,
        "user@example.com",
        "testuser",
        "password123",
        "Test User",
        false,
    )
    .await;

    // Login
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "user@example.com",
                "password": "password123"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["token"].is_string());
    assert_eq!(response["user"]["email"], "user@example.com");
    assert_eq!(response["user"]["username"], "testuser");

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_login_wrong_password() {
    let ctx = TestContext::new().await;

    // Create user
    create_test_user(
        &ctx.db,
        "user@example.com",
        "testuser",
        "correctpassword",
        "Test User",
        false,
    )
    .await;

    // Login with wrong password
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "user@example.com",
                "password": "wrongpassword"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(response["error"].is_string());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let ctx = TestContext::new().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "nonexistent@example.com",
                "password": "password123"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(response["error"].is_string());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_protected_route_without_token() {
    let ctx = TestContext::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/talks/mine")
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_protected_route_with_valid_token() {
    let ctx = TestContext::new().await;

    // Create user
    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "testuser",
        "password",
        "Test User",
        false,
    )
    .await;

    let token = generate_test_token(user_id, "user@example.com", false);

    let req = Request::builder()
        .method("GET")
        .uri("/api/talks/mine")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["talks"].is_array());

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_protected_route_with_invalid_token() {
    let ctx = TestContext::new().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/talks/mine")
        .header("authorization", "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_organizer_route_requires_organizer_role() {
    let ctx = TestContext::new().await;

    // Create regular user (not organizer)
    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "testuser",
        "password",
        "Test User",
        false, // not organizer
    )
    .await;

    let token = generate_test_token(user_id, "user@example.com", false);

    let req = Request::builder()
        .method("GET")
        .uri("/api/talks")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup().await;
}

#[tokio::test]
async fn test_organizer_route_with_organizer_role() {
    let ctx = TestContext::new().await;

    // Create organizer user
    let user_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true, // is organizer
    )
    .await;

    let token = generate_test_token(user_id, "organizer@example.com", true);

    let req = Request::builder()
        .method("GET")
        .uri("/api/talks")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["talks"].is_array());

    ctx.cleanup().await;
}
