mod e2e;

use e2e::{cleanup_database, setup_database, E2eContext};
use sqlx::postgres::PgPoolOptions;

/// Helper to create an organizer user directly in database
async fn create_organizer_user(
    email: &str,
    username: &str,
    password: &str,
    full_name: &str,
) -> Result<uuid::Uuid, Box<dyn std::error::Error>> {
    let db_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/call_for_papers_test".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // Use the same password hashing from the application
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

    let user_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO users (email, username, password_hash, full_name, is_organizer)
        VALUES ($1, $2, $3, $4, true)
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(username)
    .bind(password_hash)
    .bind(full_name)
    .fetch_one(&pool)
    .await?;

    Ok(user_id)
}

#[tokio::test]
#[ignore] // Requires WebDriver and running application
async fn test_organizer_login_and_view_dashboard() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    // Create organizer user
    create_organizer_user(
        "organizer@example.com",
        "organizer",
        "SecurePass123!",
        "Test Organizer",
    )
    .await
    .expect("Failed to create organizer");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Login as organizer
    ctx.login("organizer@example.com", "SecurePass123!")
        .await
        .expect("Failed to login");

    // Navigate to organizer dashboard
    ctx.goto("/organizer")
        .await
        .expect("Failed to navigate to organizer dashboard");

    // Verify organizer-specific elements are present
    let title = ctx.text("h1").await.expect("Failed to get page heading");

    assert!(
        title.contains("Organizer") || title.contains("Dashboard"),
        "Should show organizer dashboard"
    );

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_organizer_view_all_talks() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    // Create organizer
    create_organizer_user(
        "organizer@example.com",
        "organizer",
        "SecurePass123!",
        "Test Organizer",
    )
    .await
    .expect("Failed to create organizer");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // First, create a speaker and submit a talk
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register speaker");

    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk");
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    ctx.fill_input("#title", "Test Talk for Review")
        .await
        .expect("Failed to fill title");
    ctx.fill_input("#short_summary", "A talk to be reviewed by organizer")
        .await
        .expect("Failed to fill summary");
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    // Logout speaker
    ctx.logout().await.expect("Failed to logout");

    // Login as organizer
    ctx.login("organizer@example.com", "SecurePass123!")
        .await
        .expect("Failed to login as organizer");

    // View all talks
    ctx.goto("/organizer/talks")
        .await
        .expect("Failed to navigate to all talks");

    // Verify talk is visible
    let talks = ctx
        .find_all(".talk-item")
        .await
        .expect("Failed to find talks");

    assert_eq!(talks.len(), 1, "Should see 1 submitted talk");

    let talk_title = ctx
        .text(".talk-title")
        .await
        .expect("Failed to get talk title");

    assert_eq!(talk_title, "Test Talk for Review");

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_organizer_rate_talk() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    // Create organizer
    create_organizer_user(
        "organizer@example.com",
        "organizer",
        "SecurePass123!",
        "Test Organizer",
    )
    .await
    .expect("Failed to create organizer");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Create speaker and submit talk
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register speaker");

    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk");
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    ctx.fill_input("#title", "Talk to Rate")
        .await
        .expect("Failed to fill title");
    ctx.fill_input("#short_summary", "Summary of talk to rate")
        .await
        .expect("Failed to fill summary");
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    ctx.logout().await.expect("Failed to logout");

    // Login as organizer
    ctx.login("organizer@example.com", "SecurePass123!")
        .await
        .expect("Failed to login as organizer");

    // Navigate to talk details
    ctx.goto("/organizer/talks")
        .await
        .expect("Failed to navigate to talks");

    // Click on talk to view details
    ctx.click(".talk-item").await.expect("Failed to click talk");

    // Wait for rating form
    ctx.wait_for(".rating-form", 5)
        .await
        .expect("Failed to find rating form");

    // Select rating (assuming 1-5 scale with radio buttons or select)
    ctx.click("input[value='5']")
        .await
        .or_else(|_| async { ctx.fill_input("#rating-score", "5").await })
        .expect("Failed to select rating");

    // Add notes
    ctx.fill_input("#rating-notes", "Excellent talk proposal!")
        .await
        .expect("Failed to fill rating notes");

    // Submit rating
    ctx.click(".submit-rating-button")
        .await
        .expect("Failed to submit rating");

    ctx.wait_for_navigation(1)
        .await
        .expect("Failed to wait for navigation");

    // Verify rating was saved
    let rating_display = ctx
        .text(".my-rating")
        .await
        .expect("Failed to get rating display");

    assert!(rating_display.contains("5") || rating_display.contains("★"));

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_organizer_add_label_to_talk() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    // Create organizer
    create_organizer_user(
        "organizer@example.com",
        "organizer",
        "SecurePass123!",
        "Test Organizer",
    )
    .await
    .expect("Failed to create organizer");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Create speaker and submit talk
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register speaker");

    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk");
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    ctx.fill_input("#title", "Rust Programming Talk")
        .await
        .expect("Failed to fill title");
    ctx.fill_input("#short_summary", "About Rust")
        .await
        .expect("Failed to fill summary");
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    ctx.logout().await.expect("Failed to logout");

    // Login as organizer
    ctx.login("organizer@example.com", "SecurePass123!")
        .await
        .expect("Failed to login as organizer");

    // First create a label
    ctx.goto("/organizer/labels")
        .await
        .expect("Failed to navigate to labels");

    ctx.click(".new-label-button")
        .await
        .expect("Failed to click new label button");

    ctx.wait_for("#label-name", 5)
        .await
        .expect("Failed to find label name field");

    ctx.fill_input("#label-name", "Rust")
        .await
        .expect("Failed to fill label name");
    ctx.fill_input("#label-color", "#FF6B35")
        .await
        .expect("Failed to fill label color");

    ctx.click(".save-label-button")
        .await
        .expect("Failed to save label");

    ctx.wait_for_navigation(1)
        .await
        .expect("Failed to wait for navigation");

    // Navigate to talk and add label
    ctx.goto("/organizer/talks")
        .await
        .expect("Failed to navigate to talks");

    ctx.click(".talk-item").await.expect("Failed to click talk");

    // Add label to talk
    ctx.click(".add-label-button")
        .await
        .expect("Failed to click add label button");

    ctx.wait_for(".label-selector", 5)
        .await
        .expect("Failed to find label selector");

    ctx.click(".label-option")
        .await
        .expect("Failed to select label");

    // Verify label was added
    let labels = ctx
        .find_all(".talk-label")
        .await
        .expect("Failed to find talk labels");

    assert!(labels.len() > 0, "Should have at least one label");

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_organizer_change_talk_state() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    // Create organizer
    create_organizer_user(
        "organizer@example.com",
        "organizer",
        "SecurePass123!",
        "Test Organizer",
    )
    .await
    .expect("Failed to create organizer");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Create speaker and submit talk
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register speaker");

    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk");
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    ctx.fill_input("#title", "Talk to Accept")
        .await
        .expect("Failed to fill title");
    ctx.fill_input("#short_summary", "This talk will be accepted")
        .await
        .expect("Failed to fill summary");
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    ctx.logout().await.expect("Failed to logout");

    // Login as organizer
    ctx.login("organizer@example.com", "SecurePass123!")
        .await
        .expect("Failed to login as organizer");

    // Navigate to talk
    ctx.goto("/organizer/talks")
        .await
        .expect("Failed to navigate to talks");

    ctx.click(".talk-item").await.expect("Failed to click talk");

    // Change state to pending
    ctx.wait_for(".state-selector", 5)
        .await
        .expect("Failed to find state selector");

    ctx.click(".state-option[value='pending']")
        .await
        .or_else(|_| async { ctx.fill_input("#talk-state", "pending").await })
        .expect("Failed to select pending state");

    ctx.click(".save-state-button")
        .await
        .expect("Failed to save state change");

    ctx.wait_for_navigation(1)
        .await
        .expect("Failed to wait for navigation");

    // Verify state changed
    let status = ctx
        .text(".talk-status")
        .await
        .expect("Failed to get status");

    assert!(
        status.contains("Pending") || status.contains("pending"),
        "Status should be 'Pending'"
    );

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_organizer_create_schedule() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    // Create organizer
    create_organizer_user(
        "organizer@example.com",
        "organizer",
        "SecurePass123!",
        "Test Organizer",
    )
    .await
    .expect("Failed to create organizer");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Login as organizer
    ctx.login("organizer@example.com", "SecurePass123!")
        .await
        .expect("Failed to login as organizer");

    // Navigate to schedule builder
    ctx.goto("/organizer/schedule")
        .await
        .expect("Failed to navigate to schedule");

    // Create conference
    ctx.click(".new-conference-button")
        .await
        .expect("Failed to click new conference");

    ctx.wait_for("#conference-name", 5)
        .await
        .expect("Failed to find conference name field");

    ctx.fill_input("#conference-name", "TXLF 2025")
        .await
        .expect("Failed to fill conference name");
    ctx.fill_input("#conference-description", "Texas Linux Fest 2025")
        .await
        .expect("Failed to fill description");
    ctx.fill_input("#start-date", "2025-04-18")
        .await
        .expect("Failed to fill start date");
    ctx.fill_input("#end-date", "2025-04-20")
        .await
        .expect("Failed to fill end date");
    ctx.fill_input("#location", "Austin, TX")
        .await
        .expect("Failed to fill location");

    ctx.click(".save-conference-button")
        .await
        .expect("Failed to save conference");

    ctx.wait_for_navigation(1)
        .await
        .expect("Failed to wait for navigation");

    // Verify conference was created
    let conference_title = ctx
        .text(".conference-title")
        .await
        .expect("Failed to get conference title");

    assert_eq!(conference_title, "TXLF 2025");

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}
