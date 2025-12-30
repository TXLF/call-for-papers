use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::Config;

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

pub struct ClaudeService {
    api_key: Option<String>,
    client: Client,
}

impl ClaudeService {
    pub fn new(config: &Config) -> Self {
        Self {
            api_key: config.claude_api_key.clone(),
            client: Client::new(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    /// Generate label suggestions for talks using Claude API
    pub async fn suggest_labels(&self, talks_json: &str) -> Result<Vec<String>, String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or("Claude API key not configured")?;

        let prompt = format!(
            r#"You are analyzing talk proposals for a technical conference. Based on the following talk submissions, suggest relevant labels/tags that could be used to categorize them.

Talk Submissions:
{}

Please analyze these talks and suggest 10-15 relevant labels that would help organizers categorize and filter submissions. Consider:
- Technical topics and technologies mentioned
- Talk formats (workshop, tutorial, case study, etc.)
- Skill levels (beginner, intermediate, advanced)
- Themes and domains

Return ONLY a JSON array of label strings, like: ["label1", "label2", "label3"]
Do not include any other text or explanation."#,
            talks_json
        );

        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to call Claude API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Claude API returned error {}: {}",
                status, error_text
            ));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

        let text = claude_response
            .content
            .first()
            .ok_or("No content in Claude response")?
            .text
            .clone();

        // Parse the JSON array from Claude's response
        let labels: Vec<String> = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse labels from Claude response: {}", e))?;

        Ok(labels)
    }
}
