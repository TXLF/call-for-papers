use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

#[derive(Clone)]
pub struct OpenAIService {
    api_key: Option<String>,
    client: Client,
}

impl OpenAIService {
    pub fn new(config: &Config) -> Self {
        Self {
            api_key: config.openai_api_key.clone(),
            client: Client::new(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    /// Generate label suggestions for talks using OpenAI API
    pub async fn suggest_labels(&self, talks_json: &str) -> Result<Vec<String>, String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or("OpenAI API key not configured")?;

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

        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to call OpenAI API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "OpenAI API returned error {}: {}",
                status, error_text
            ));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        let text = openai_response
            .choices
            .first()
            .ok_or("No choices in OpenAI response")?
            .message
            .content
            .clone();

        // Parse the JSON array from OpenAI's response
        let labels: Vec<String> = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse labels from OpenAI response: {}", e))?;

        Ok(labels)
    }
}
