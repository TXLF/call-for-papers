mod common;
use serial_test::serial;


use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::json;

#[tokio::test]
#[serial]
async fn test_create_talk_success() {
    let ctx = TestContext::new().await;

    // Create speaker
    let user_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let token = generate_test_token(user_id, "speaker@example.com", false);

    let req = Request::builder()
        .method("POST")
        .uri("/api/talks")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "title": "Introduction to Rust",
                "short_summary": "Learn the basics of Rust programming",
                "long_description": "This talk covers Rust fundamentals including ownership, borrowing, and lifetimes.",
                "label_ids": []
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].is_string());
    assert_eq!(response["title"], "Introduction to Rust");
    assert_eq!(
        response["short_summary"],
        "Learn the basics of Rust programming"
    );
    assert_eq!(response["speaker_id"], user_id.to_string());
    assert_eq!(response["state"], "submitted");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_talk_without_auth() {
    let ctx = TestContext::new().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/talks")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "title": "Introduction to Rust",
                "short_summary": "Learn the basics of Rust programming"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_talk_missing_required_fields() {
    let ctx = TestContext::new().await;

    let user_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let token = generate_test_token(user_id, "speaker@example.com", false);

    // Missing short_summary
    let req = Request::builder()
        .method("POST")
        .uri("/api/talks")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "title": "Introduction to Rust"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_list_my_talks() {
    let ctx = TestContext::new().await;

    // Create speaker and talks
    let user_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    create_test_talk(&ctx.db, user_id, "Talk 1", "Summary 1").await;

    create_test_talk(&ctx.db, user_id, "Talk 2", "Summary 2").await;

    let token = generate_test_token(user_id, "speaker@example.com", false);

    let req = Request::builder()
        .method("GET")
        .uri("/api/talks/mine")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["talks"].is_array());
    let talks = response["talks"].as_array().unwrap();
    assert_eq!(talks.len(), 2);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_get_talk_by_id() {
    let ctx = TestContext::new().await;

    let user_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, user_id, "Test Talk", "Test Summary").await;

    let token = generate_test_token(user_id, "speaker@example.com", false);

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/talks/{}", talk_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"], talk_id.to_string());
    assert_eq!(response["title"], "Test Talk");
    assert_eq!(response["short_summary"], "Test Summary");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_update_talk() {
    let ctx = TestContext::new().await;

    let user_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, user_id, "Original Title", "Original Summary").await;

    let token = generate_test_token(user_id, "speaker@example.com", false);

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/talks/{}", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "title": "Updated Title",
                "short_summary": "Updated Summary",
                "long_description": "Updated description"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["title"], "Updated Title");
    assert_eq!(response["short_summary"], "Updated Summary");
    assert_eq!(response["long_description"], "Updated description");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_update_talk_unauthorized() {
    let ctx = TestContext::new().await;

    // Create speaker 1 with a talk
    let user1_id = create_test_user(
        &ctx.db,
        "speaker1@example.com",
        "speaker1",
        "password",
        "Speaker 1",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, user1_id, "Original Title", "Original Summary").await;

    // Create speaker 2 who tries to update speaker 1's talk
    let user2_id = create_test_user(
        &ctx.db,
        "speaker2@example.com",
        "speaker2",
        "password",
        "Speaker 2",
        false,
    )
    .await;

    let token = generate_test_token(user2_id, "speaker2@example.com", false);

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/talks/{}", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "title": "Hacked Title",
                "short_summary": "Hacked Summary"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_delete_talk() {
    let ctx = TestContext::new().await;

    let user_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, user_id, "Talk to Delete", "Summary").await;

    let token = generate_test_token(user_id, "speaker@example.com", false);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/talks/{}", talk_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify talk is deleted
    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/talks/{}", talk_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_list_all_talks_as_organizer() {
    let ctx = TestContext::new().await;

    // Create speakers and talks
    let speaker1_id = create_test_user(
        &ctx.db,
        "speaker1@example.com",
        "speaker1",
        "password",
        "Speaker 1",
        false,
    )
    .await;

    let speaker2_id = create_test_user(
        &ctx.db,
        "speaker2@example.com",
        "speaker2",
        "password",
        "Speaker 2",
        false,
    )
    .await;

    create_test_talk(&ctx.db, speaker1_id, "Talk 1", "Summary 1").await;
    create_test_talk(&ctx.db, speaker2_id, "Talk 2", "Summary 2").await;

    // Create organizer
    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(organizer_id, "organizer@example.com", true);

    let req = Request::builder()
        .method("GET")
        .uri("/api/talks")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["talks"].is_array());
    let talks = response["talks"].as_array().unwrap();
    assert_eq!(talks.len(), 2);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_list_all_talks_requires_organizer() {
    let ctx = TestContext::new().await;

    // Regular user (not organizer)
    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "Regular User",
        false,
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
#[serial]
async fn test_change_talk_state_as_organizer() {
    let ctx = TestContext::new().await;

    // Create speaker and talk
    let speaker_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, speaker_id, "Test Talk", "Test Summary").await;

    // Create organizer
    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(organizer_id, "organizer@example.com", true);

    // Change state to pending
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/talks/{}/state", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "state": "pending"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["state"], "pending");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_change_talk_state_requires_organizer() {
    let ctx = TestContext::new().await;

    // Create speaker and talk
    let speaker_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, speaker_id, "Test Talk", "Test Summary").await;

    let token = generate_test_token(speaker_id, "speaker@example.com", false);

    // Try to change state as speaker (should fail)
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/talks/{}/state", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "state": "accepted"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_respond_to_talk() {
    let ctx = TestContext::new().await;

    // Create speaker and talk
    let speaker_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, speaker_id, "Test Talk", "Test Summary").await;

    // Set talk to pending state first (simulating organizer acceptance)
    sqlx::query("UPDATE talks SET state = 'pending' WHERE id = $1")
        .bind(talk_id)
        .execute(&ctx.db)
        .await
        .unwrap();

    let token = generate_test_token(speaker_id, "speaker@example.com", false);

    // Accept the talk
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/talks/{}/respond", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "accepted": true
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["state"], "accepted");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_respond_to_talk_reject() {
    let ctx = TestContext::new().await;

    // Create speaker and talk
    let speaker_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, speaker_id, "Test Talk", "Test Summary").await;

    // Set talk to pending state
    sqlx::query("UPDATE talks SET state = 'pending' WHERE id = $1")
        .bind(talk_id)
        .execute(&ctx.db)
        .await
        .unwrap();

    let token = generate_test_token(speaker_id, "speaker@example.com", false);

    // Decline the talk
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/talks/{}/respond", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "accepted": false
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["state"], "rejected");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_respond_to_talk_wrong_state() {
    let ctx = TestContext::new().await;

    // Create speaker and talk
    let speaker_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, speaker_id, "Test Talk", "Test Summary").await;

    // Talk is in 'submitted' state, not 'pending'
    let token = generate_test_token(speaker_id, "speaker@example.com", false);

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/talks/{}/respond", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "accepted": true
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    ctx.cleanup().await;
}
