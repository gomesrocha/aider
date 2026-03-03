use super::LlmProvider;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct GeminiProvider {
    api_key: String,
    model: String,
    client: Client,
}

impl GeminiProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let payload = json!({
            "systemInstruction": {
                "parts": [{"text": system_prompt}]
            },
            "contents": [{
                "parts": [{"text": user_prompt}]
            }],
            "generationConfig": {
                "temperature": 0.0
            }
        });

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to Gemini")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            anyhow::bail!("Gemini API error: {}", error_text);
        }

        let body: serde_json::Value = res.json().await.context("Failed to parse response")?;

        let content = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in response"))?;

        Ok(content.to_string())
    }
}
