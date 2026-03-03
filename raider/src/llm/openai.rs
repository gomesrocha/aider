use super::LlmProvider;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let payload = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.0
        });

        let res = self.client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            anyhow::bail!("OpenAI API error: {}", error_text);
        }

        let body: serde_json::Value = res.json().await.context("Failed to parse response")?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in response"))?;

        Ok(content.to_string())
    }
}
