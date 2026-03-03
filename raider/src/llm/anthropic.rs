use super::LlmProvider;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let payload = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ]
        });

        let res = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            anyhow::bail!("Anthropic API error: {}", error_text);
        }

        let body: serde_json::Value = res.json().await.context("Failed to parse response")?;

        let content = body["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in response"))?;

        Ok(content.to_string())
    }
}
