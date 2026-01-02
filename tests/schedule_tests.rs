mod common;
use serial_test::serial;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use serde_json::json;

// ============================================================================
// Conference Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_create_conference_as_organizer() {
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

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/conferences")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Texas Linux Fest 2025",
                "description": "Annual open source conference",
                "start_date": "2025-04-18",
                "end_date": "2025-04-20",
                "location": "Austin, TX",
                "is_active": true
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].is_string());
    assert_eq!(response["name"], "Texas Linux Fest 2025");
    assert_eq!(response["location"], "Austin, TX");
    assert_eq!(response["is_active"], true);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_conference_requires_organizer() {
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

    let token = generate_test_token(&ctx.db, user_id, "user@example.com", false).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/conferences")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Unauthorized Conference",
                "description": "This should fail",
                "start_date": "2025-04-18",
                "end_date": "2025-04-20",
                "location": "Austin, TX"
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
async fn test_list_conferences() {
    let ctx = TestContext::new().await;

    create_test_conference(&ctx.db, "Conference 1").await;
    create_test_conference(&ctx.db, "Conference 2").await;

    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "User",
        false,
    )
    .await;

    let token = generate_test_token(&ctx.db, user_id, "user@example.com", false).await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/conferences")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["conferences"].is_array());
    let conferences = response["conferences"].as_array().unwrap();
    assert_eq!(conferences.len(), 2);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_get_conference_by_id() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;

    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "User",
        false,
    )
    .await;

    let token = generate_test_token(&ctx.db, user_id, "user@example.com", false).await;

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/conferences/{}", conference_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["id"], conference_id.to_string());
    assert_eq!(response["name"], "Test Conference");

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_update_conference() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Original Conference").await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/conferences/{}", conference_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Updated Conference",
                "description": "Updated description",
                "start_date": "2025-04-18",
                "end_date": "2025-04-20",
                "location": "Updated Location",
                "is_active": false
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"], "Updated Conference");
    assert_eq!(response["location"], "Updated Location");
    assert_eq!(response["is_active"], false);

    ctx.cleanup().await;
}

// ============================================================================
// Track Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_create_track_as_organizer() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/tracks")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "conference_id": conference_id,
                "name": "Main Hall",
                "capacity": 200
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].is_string());
    assert_eq!(response["name"], "Main Hall");
    assert_eq!(response["capacity"], 200);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_track_requires_organizer() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;

    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "Regular User",
        false,
    )
    .await;

    let token = generate_test_token(&ctx.db, user_id, "user@example.com", false).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/tracks")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "conference_id": conference_id,
                "name": "Unauthorized Track",
                "capacity": 100
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
async fn test_list_tracks_for_conference() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;

    create_test_track(&ctx.db, conference_id, "Track 1").await;
    create_test_track(&ctx.db, conference_id, "Track 2").await;

    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "User",
        false,
    )
    .await;

    let token = generate_test_token(&ctx.db, user_id, "user@example.com", false).await;

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/conferences/{}/tracks", conference_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["tracks"].is_array());
    let tracks = response["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_update_track() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Original Track").await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/tracks/{}", track_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "name": "Updated Track",
                "capacity": 150
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["name"], "Updated Track");
    assert_eq!(response["capacity"], 150);

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_delete_track() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Track to Delete").await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/tracks/{}", track_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    ctx.cleanup().await;
}

// ============================================================================
// Schedule Slot Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_create_schedule_slot() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Main Track").await;

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/schedule-slots")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "track_id": track_id,
                "start_time": "2025-04-18T09:00:00Z",
                "end_time": "2025-04-18T10:00:00Z"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].is_string());
    assert_eq!(response["track_id"], track_id.to_string());

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_create_schedule_slot_requires_organizer() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Main Track").await;

    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "Regular User",
        false,
    )
    .await;

    let token = generate_test_token(&ctx.db, user_id, "user@example.com", false).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/schedule-slots")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "track_id": track_id,
                "start_time": "2025-04-18T09:00:00Z",
                "end_time": "2025-04-18T10:00:00Z"
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
async fn test_assign_talk_to_slot() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Main Track").await;

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

    // Set talk to accepted state
    sqlx::query("UPDATE talks SET state = 'accepted' WHERE id = $1")
        .bind(talk_id)
        .execute(&ctx.db)
        .await
        .unwrap();

    // Create slot
    let slot_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO schedule_slots (track_id, start_time, end_time) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(track_id)
    .bind("2025-04-18T09:00:00Z")
    .bind("2025-04-18T10:00:00Z")
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/schedule-slots/{}/talk", slot_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(
            json!({
                "talk_id": talk_id
            })
            .to_string(),
        ))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["talk_id"], talk_id.to_string());

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_unassign_talk_from_slot() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Main Track").await;

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

    // Create slot with assigned talk
    let slot_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO schedule_slots (track_id, start_time, end_time, talk_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(track_id)
    .bind("2025-04-18T09:00:00Z")
    .bind("2025-04-18T10:00:00Z")
    .bind(talk_id)
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/schedule-slots/{}/talk", slot_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["talk_id"].is_null());

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_get_conference_schedule() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Main Track").await;

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

    // Create slots with assigned talks
    sqlx::query(
        "INSERT INTO schedule_slots (track_id, start_time, end_time, talk_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(track_id)
    .bind("2025-04-18T09:00:00Z")
    .bind("2025-04-18T10:00:00Z")
    .bind(talk_id)
    .execute(&ctx.db)
    .await
    .unwrap();

    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "user",
        "password",
        "User",
        false,
    )
    .await;

    let token = generate_test_token(&ctx.db, user_id, "user@example.com", false).await;

    let req = Request::builder()
        .method("GET")
        .uri(format!("/api/conferences/{}/schedule", conference_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["schedule"].is_array());

    ctx.cleanup().await;
}

#[tokio::test]
#[serial]
async fn test_delete_schedule_slot() {
    let ctx = TestContext::new().await;

    let conference_id = create_test_conference(&ctx.db, "Test Conference").await;
    let track_id = create_test_track(&ctx.db, conference_id, "Main Track").await;

    let slot_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO schedule_slots (track_id, start_time, end_time) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(track_id)
    .bind("2025-04-18T09:00:00Z")
    .bind("2025-04-18T10:00:00Z")
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    let organizer_id = create_test_user(
        &ctx.db,
        "organizer@example.com",
        "organizer",
        "password",
        "Organizer User",
        true,
    )
    .await;

    let token = generate_test_token(&ctx.db, organizer_id, "organizer@example.com", true).await;

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/schedule-slots/{}", slot_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    ctx.cleanup().await;
}
