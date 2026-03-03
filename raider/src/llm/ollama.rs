use super::LlmProvider;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OllamaProvider {
    model: String,
    client: Client,
}

impl OllamaProvider {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let payload = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "stream": false,
            "options": {
                "temperature": 0.0
            }
        });

        let res = self.client.post("http://127.0.0.1:11434/api/chat")
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to Ollama (make sure Ollama is running)")?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            anyhow::bail!("Ollama API error: {}", error_text);
        }

        let body: serde_json::Value = res.json().await.context("Failed to parse response")?;

        let content = body["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in response"))?;

        Ok(content.to_string())
    }
}
