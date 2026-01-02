mod common;
use serial_test::serial;


use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::json;

// ============================================================================
// Label Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_create_label_as_organizer() {
    let ctx = TestContext::new().await;

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
        .method("POST")
        .uri("/api/labels")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Rust",
                "color": "#FF6B35"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].is_string());
    assert_eq!(response["name"], "Rust");
    assert_eq!(response["color"], "#FF6B35");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_label_requires_organizer() {
    let ctx = TestContext::new().await;

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
        .method("POST")
        .uri("/api/labels")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Rust",
                "color": "#FF6B35"
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
async fn test_list_labels() {
    let ctx = TestContext::new().await;

    // Create labels
    create_test_label(&ctx.db, "Rust", "#FF6B35").await;
    create_test_label(&ctx.db, "Go", "#00ADD8").await;
    create_test_label(&ctx.db, "Python", "#3776AB").await;

    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "User",
        false,
    )
    .await;

    let token = generate_test_token(user_id, "user@example.com", false);

    let req = Request::builder()
        .method("GET")
        .uri("/api/labels")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["labels"].is_array());
    let labels = response["labels"].as_array().unwrap();
    assert_eq!(labels.len(), 3);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_update_label_as_organizer() {
    let ctx = TestContext::new().await;

    let label_id = create_test_label(&ctx.db, "Rust", "#FF6B35").await;

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
        .method("PUT")
        .uri(format!("/api/labels/{}", label_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Rust Programming",
                "color": "#FF0000"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"], "Rust Programming");
    assert_eq!(response["color"], "#FF0000");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_delete_label_as_organizer() {
    let ctx = TestContext::new().await;

    let label_id = create_test_label(&ctx.db, "Rust", "#FF6B35").await;

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
        .method("DELETE")
        .uri(format!("/api/labels/{}", label_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_add_label_to_talk() {
    let ctx = TestContext::new().await;

    let label_id = create_test_label(&ctx.db, "Rust", "#FF6B35").await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let speaker_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, speaker_id, "Rust Talk", "Learn Rust").await;

    let token = generate_test_token(organizer_id, "organizer@example.com", true);

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/talks/{}/labels", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "label_id": label_id
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_remove_label_from_talk() {
    let ctx = TestContext::new().await;

    let label_id = create_test_label(&ctx.db, "Rust", "#FF6B35").await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let speaker_id = create_test_user(
        &ctx.db,
        "speaker@example.com",
        "speaker",
        "password",
        "Speaker User",
        false,
    )
    .await;

    let talk_id = create_test_talk(&ctx.db, speaker_id, "Rust Talk", "Learn Rust").await;

    // Add label to talk
    sqlx::query("INSERT INTO talk_labels (talk_id, label_id) VALUES ($1, $2)")
        .bind(talk_id)
        .bind(label_id)
        .execute(&ctx.db)
        .await
        .unwrap();

    let token = generate_test_token(organizer_id, "organizer@example.com", true);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/talks/{}/labels/{}", talk_id, label_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    ctx.cleanup().await;
}

// ============================================================================
// Rating Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_create_rating_as_organizer() {
    let ctx = TestContext::new().await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

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

    let token = generate_test_token(organizer_id, "organizer@example.com", true);

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/talks/{}/ratings", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "score": 5,
                "notes": "Excellent talk proposal"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].is_string());
    assert_eq!(response["score"], 5);
    assert_eq!(response["notes"], "Excellent talk proposal");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_rating_requires_organizer() {
    let ctx = TestContext::new().await;

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

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/talks/{}/ratings", talk_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "score": 5,
                "notes": "Self rating"
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
async fn test_update_rating() {
    let ctx = TestContext::new().await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

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

    // Create rating
    let rating_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO ratings (talk_id, organizer_id, score, notes) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(talk_id)
    .bind(organizer_id)
    .bind(3)
    .bind("Initial rating")
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    let token = generate_test_token(organizer_id, "organizer@example.com", true);

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/ratings/{}", rating_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "score": 5,
                "notes": "Updated: Excellent talk"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["score"], 5);
    assert_eq!(response["notes"], "Updated: Excellent talk");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_update_rating_only_own() {
    let ctx = TestContext::new().await;

    let organizer1_id = create_test_user(
        &ctx.db,
        "organizer1@example.com",
        "organizer1",
        "password",
        "Organizer 1",
        true,
    )
    .await;

    let organizer2_id = create_test_user(
        &ctx.db,
        "organizer2@example.com",
        "organizer2",
        "password",
        "Organizer 2",
        true,
    )
    .await;

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

    // Create rating by organizer1
    let rating_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO ratings (talk_id, organizer_id, score, notes) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(talk_id)
    .bind(organizer1_id)
    .bind(3)
    .bind("Organizer 1 rating")
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    // Try to update with organizer2's token
    let token = generate_test_token(organizer2_id, "organizer2@example.com", true);

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/ratings/{}", rating_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "score": 1,
                "notes": "Hacked rating"
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
async fn test_delete_rating() {
    let ctx = TestContext::new().await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

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

    // Create rating
    let rating_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO ratings (talk_id, organizer_id, score, notes) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(talk_id)
    .bind(organizer_id)
    .bind(3)
    .bind("Rating to delete")
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    let token = generate_test_token(organizer_id, "organizer@example.com", true);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/ratings/{}", rating_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_list_ratings_for_talk() {
    let ctx = TestContext::new().await;

    let organizer1_id = create_test_user(
        &ctx.db,
        "organizer1@example.com",
        "organizer1",
        "password",
        "Organizer 1",
        true,
    )
    .await;

    let organizer2_id = create_test_user(
        &ctx.db,
        "organizer2@example.com",
        "organizer2",
        "password",
        "Organizer 2",
        true,
    )
    .await;

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

    // Create ratings
    sqlx::query(
        "INSERT INTO ratings (talk_id, organizer_id, score, notes) VALUES ($1, $2, $3, $4)",
    )
    .bind(talk_id)
    .bind(organizer1_id)
    .bind(5)
    .bind("Great")
    .execute(&ctx.db)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ratings (talk_id, organizer_id, score, notes) VALUES ($1, $2, $3, $4)",
    )
    .bind(talk_id)
    .bind(organizer2_id)
    .bind(4)
    .bind("Good")
    .execute(&ctx.db)
    .await
    .unwrap();

    let token = generate_test_token(organizer1_id, "organizer1@example.com", true);

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/talks/{}/ratings", talk_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["ratings"].is_array());
    let ratings = response["ratings"].as_array().unwrap();
    assert_eq!(ratings.len(), 2);
    assert!(response["average_score"].is_number());

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_list_ratings_requires_organizer() {
    let ctx = TestContext::new().await;

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

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/talks/{}/ratings", talk_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup().await;
}
