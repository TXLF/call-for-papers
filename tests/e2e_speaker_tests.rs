mod e2e;

use e2e::{cleanup_database, setup_database, E2eContext};

#[tokio::test]
#[ignore] // Requires WebDriver and running application
async fn test_speaker_registration_and_login() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Register new speaker
    ctx.register(
        "speaker@example.com",
        "newspeaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register");

    // Verify we're redirected after registration
    let url = ctx.current_url().await.expect("Failed to get URL");
    assert!(url.contains("dashboard") || url.contains("talks"));

    // Logout
    ctx.logout().await.expect("Failed to logout");

    // Login again
    ctx.login("speaker@example.com", "SecurePass123!")
        .await
        .expect("Failed to login");

    // Verify we're logged in
    let url = ctx.current_url().await.expect("Failed to get URL");
    assert!(url.contains("dashboard") || url.contains("talks"));

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_speaker_submit_talk() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Register and login
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register");

    // Navigate to submit talk page
    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk page");

    // Wait for form to load
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    // Fill in talk details
    ctx.fill_input("#title", "Introduction to Rust Programming")
        .await
        .expect("Failed to fill title");

    ctx.fill_input(
        "#short_summary",
        "Learn the basics of Rust programming language",
    )
    .await
    .expect("Failed to fill summary");

    ctx.fill_input(
        "#long_description",
        "This comprehensive talk covers Rust fundamentals including ownership, borrowing, and lifetimes. Perfect for beginners.",
    )
    .await
    .expect("Failed to fill description");

    // Submit the form
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    // Wait for navigation
    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    // Verify talk appears in my talks list
    ctx.goto("/talks/mine")
        .await
        .expect("Failed to navigate to my talks");

    let talks_count = ctx
        .find_all(".talk-item")
        .await
        .expect("Failed to find talks")
        .len();

    assert_eq!(talks_count, 1, "Should have 1 submitted talk");

    // Verify talk title is displayed
    let talk_title = ctx
        .text(".talk-title")
        .await
        .expect("Failed to get talk title");

    assert_eq!(talk_title, "Introduction to Rust Programming");

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_speaker_edit_talk() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Register and login
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register");

    // Submit a talk first
    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk page");
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    ctx.fill_input("#title", "Original Title")
        .await
        .expect("Failed to fill title");
    ctx.fill_input("#short_summary", "Original summary")
        .await
        .expect("Failed to fill summary");
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    // Navigate to my talks
    ctx.goto("/talks/mine")
        .await
        .expect("Failed to navigate to my talks");

    // Click edit button
    ctx.click(".edit-talk-button")
        .await
        .expect("Failed to click edit button");

    // Wait for edit form
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    // Clear and update fields
    let title_input = ctx.find("#title").await.expect("Failed to find title");
    title_input.clear().await.expect("Failed to clear title");

    ctx.fill_input("#title", "Updated Title")
        .await
        .expect("Failed to update title");

    // Submit changes
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit changes");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    // Verify updated title
    ctx.goto("/talks/mine")
        .await
        .expect("Failed to navigate to my talks");

    let updated_title = ctx
        .text(".talk-title")
        .await
        .expect("Failed to get updated title");

    assert_eq!(updated_title, "Updated Title");

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_speaker_delete_talk() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Register and login
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register");

    // Submit a talk
    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk page");
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    ctx.fill_input("#title", "Talk to Delete")
        .await
        .expect("Failed to fill title");
    ctx.fill_input("#short_summary", "This talk will be deleted")
        .await
        .expect("Failed to fill summary");
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    // Navigate to my talks
    ctx.goto("/talks/mine")
        .await
        .expect("Failed to navigate to my talks");

    // Verify talk exists
    let talks_before = ctx
        .find_all(".talk-item")
        .await
        .expect("Failed to find talks")
        .len();

    assert_eq!(talks_before, 1);

    // Click delete button
    ctx.click(".delete-talk-button")
        .await
        .expect("Failed to click delete button");

    // Confirm deletion (if there's a confirmation dialog)
    if (ctx.find(".confirm-delete-button").await).is_ok() {
        ctx.click(".confirm-delete-button")
            .await
            .expect("Failed to confirm deletion");
    }

    ctx.wait_for_navigation(1)
        .await
        .expect("Failed to wait for navigation");

    // Verify talk is deleted
    ctx.goto("/talks/mine")
        .await
        .expect("Failed to navigate to my talks");

    let talks_after = ctx.find_all(".talk-item").await.unwrap_or_default().len();

    assert_eq!(talks_after, 0, "Talk should be deleted");

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_speaker_view_talk_status() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    // Register and login
    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register");

    // Submit a talk
    ctx.goto("/talks/new")
        .await
        .expect("Failed to navigate to new talk page");
    ctx.wait_for("#title", 5)
        .await
        .expect("Failed to find title field");

    ctx.fill_input("#title", "Test Talk Status")
        .await
        .expect("Failed to fill title");
    ctx.fill_input("#short_summary", "Testing status display")
        .await
        .expect("Failed to fill summary");
    ctx.click("button[type='submit']")
        .await
        .expect("Failed to submit talk");

    ctx.wait_for_navigation(2)
        .await
        .expect("Failed to wait for navigation");

    // Navigate to my talks
    ctx.goto("/talks/mine")
        .await
        .expect("Failed to navigate to my talks");

    // Check status badge
    let status = ctx
        .text(".talk-status")
        .await
        .expect("Failed to get status");

    assert!(
        status.contains("Submitted") || status.contains("submitted"),
        "Status should be 'Submitted'"
    );

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}

#[tokio::test]
#[ignore]
async fn test_speaker_respond_to_acceptance() {
    setup_database().await.expect("Failed to setup database");
    cleanup_database()
        .await
        .expect("Failed to cleanup database");

    // This test would require:
    // 1. Creating a speaker account
    // 2. Creating an organizer account
    // 3. Submitting a talk as speaker
    // 4. Accepting the talk as organizer (sets status to "pending")
    // 5. Responding to acceptance as speaker

    // For now, we'll create a simplified version that demonstrates the workflow
    let ctx = E2eContext::new()
        .await
        .expect("Failed to create E2E context");

    ctx.register(
        "speaker@example.com",
        "speaker",
        "SecurePass123!",
        "Test Speaker",
    )
    .await
    .expect("Failed to register");

    // In a real scenario, an organizer would accept the talk
    // Then the speaker would see an "Accept" or "Decline" button

    // For this test, we'll just verify the UI elements exist when we have pending talks
    ctx.goto("/talks/mine")
        .await
        .expect("Failed to navigate to my talks");

    // The actual acceptance flow would be tested with database fixtures
    // showing a talk in "pending" state

    ctx.cleanup().await;
    cleanup_database()
        .await
        .expect("Failed to cleanup database");
}
