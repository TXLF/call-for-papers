use fantoccini::{Client, ClientBuilder, Locator};
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

/// E2E test context managing browser and application server
pub struct E2eContext {
    pub client: Client,
    pub base_url: String,
    server_process: Option<Child>,
}

impl E2eContext {
    /// Create a new E2E test context with a running application
    #[allow(clippy::zombie_processes)]
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Start the application server
        println!("Starting application server...");
        let server_process = Command::new("cargo")
            .args(["run", "--release"])
            .env("DATABASE_URL", get_test_database_url())
            .env("JWT_SECRET", "test_jwt_secret_for_e2e_tests")
            .env("RUST_LOG", "info")
            .spawn()
            .expect("Failed to start application server");

        // Wait for server to start
        sleep(Duration::from_secs(5)).await;

        // Connect to WebDriver (geckodriver or chromedriver)
        let client = ClientBuilder::native()
            .connect("http://localhost:4444")
            .await?;

        let base_url = "http://localhost:8080".to_string();

        Ok(Self {
            client,
            base_url,
            server_process: Some(server_process),
        })
    }

    /// Navigate to a path relative to base URL
    pub async fn goto(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        self.client.goto(&url).await?;
        Ok(())
    }

    /// Find element by CSS selector
    pub async fn find(
        &self,
        selector: &str,
    ) -> Result<fantoccini::elements::Element, Box<dyn std::error::Error>> {
        let element = self.client.find(Locator::Css(selector)).await?;
        Ok(element)
    }

    /// Find all elements by CSS selector
    pub async fn find_all(
        &self,
        selector: &str,
    ) -> Result<Vec<fantoccini::elements::Element>, Box<dyn std::error::Error>> {
        let elements = self.client.find_all(Locator::Css(selector)).await?;
        Ok(elements)
    }

    /// Wait for element to appear
    pub async fn wait_for(
        &self,
        selector: &str,
        timeout_secs: u64,
    ) -> Result<fantoccini::elements::Element, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        loop {
            if let Ok(element) = self.find(selector).await {
                return Ok(element);
            }
            if start.elapsed() > Duration::from_secs(timeout_secs) {
                return Err(format!("Timeout waiting for element: {}", selector).into());
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Fill input field
    pub async fn fill_input(
        &self,
        selector: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let input = self.find(selector).await?;
        input.send_keys(value).await?;
        Ok(())
    }

    /// Click element
    pub async fn click(&self, selector: &str) -> Result<(), Box<dyn std::error::Error>> {
        let element = self.find(selector).await?;
        element.click().await?;
        Ok(())
    }

    /// Get text content of element
    pub async fn text(&self, selector: &str) -> Result<String, Box<dyn std::error::Error>> {
        let element = self.find(selector).await?;
        let text = element.text().await?;
        Ok(text)
    }

    /// Get page title
    pub async fn title(&self) -> Result<String, Box<dyn std::error::Error>> {
        let title = self.client.title().await?;
        Ok(title)
    }

    /// Get current URL
    pub async fn current_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.client.current_url().await?;
        Ok(url.to_string())
    }

    /// Take screenshot
    pub async fn screenshot(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let screenshot = self.client.screenshot().await?;
        std::fs::write(path, screenshot)?;
        Ok(())
    }

    /// Wait for navigation
    pub async fn wait_for_navigation(
        &self,
        timeout_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    /// Login as a user
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.goto("/login").await?;
        self.wait_for("#email", 5).await?;

        self.fill_input("#email", email).await?;
        self.fill_input("#password", password).await?;
        self.click("button[type='submit']").await?;

        self.wait_for_navigation(2).await?;
        Ok(())
    }

    /// Register a new user
    pub async fn register(
        &self,
        email: &str,
        username: &str,
        password: &str,
        full_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.goto("/register").await?;
        self.wait_for("#email", 5).await?;

        self.fill_input("#email", email).await?;
        self.fill_input("#username", username).await?;
        self.fill_input("#password", password).await?;
        self.fill_input("#full_name", full_name).await?;
        self.click("button[type='submit']").await?;

        self.wait_for_navigation(2).await?;
        Ok(())
    }

    /// Logout
    pub async fn logout(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.click("#logout-button").await?;
        self.wait_for_navigation(1).await?;
        Ok(())
    }

    /// Cleanup resources
    pub async fn cleanup(mut self) {
        // Close browser
        let _ = self.client.close().await;

        // Kill server process
        if let Some(mut process) = self.server_process.take() {
            let _ = process.kill();
        }
    }
}

/// Get test database URL
fn get_test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/call_for_papers_test".to_string()
    })
}

/// Setup database for E2E tests
pub async fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::postgres::PgPoolOptions;

    let db_url = get_test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(())
}

/// Clean database tables
pub async fn cleanup_database() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::postgres::PgPoolOptions;

    let db_url = get_test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

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
            .execute(&pool)
            .await
            .ok();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires WebDriver running
    async fn test_e2e_context_creation() {
        let ctx = E2eContext::new()
            .await
            .expect("Failed to create E2E context");

        // Navigate to home page
        ctx.goto("/").await.expect("Failed to navigate");

        let title = ctx.title().await.expect("Failed to get title");
        assert!(!title.is_empty());

        ctx.cleanup().await;
    }
}
