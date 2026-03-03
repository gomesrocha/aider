use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String>;
}

pub mod openai;
pub mod anthropic;
pub mod gemini;
pub mod ollama;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model_name: String,
    pub api_key: Option<String>,
}

pub fn create_provider(config: &ModelConfig) -> Result<Box<dyn LlmProvider>> {
    match config.provider.as_str() {
        "openai" => Ok(Box::new(openai::OpenAiProvider::new(&config.api_key.clone().unwrap_or_default(), &config.model_name))),
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(&config.api_key.clone().unwrap_or_default(), &config.model_name))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(&config.api_key.clone().unwrap_or_default(), &config.model_name))),
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(&config.model_name))),
        _ => anyhow::bail!("Unknown provider: {}", config.provider),
    }
}
