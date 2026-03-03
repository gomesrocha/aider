use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiSettings {
    pub openai_key: Option<String>,
    pub anthropic_key: Option<String>,
    pub gemini_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    ReadingFiles,
    Thinking,
    ApplyingChanges,
    Committing,
    Done,
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub target_dir: String,
    pub prompt: String,
    pub model: crate::llm::ModelConfig,
    pub status: TaskStatus,
    pub logs: Vec<String>,
}

pub struct AppState {
    pub settings: ApiSettings,
    pub tasks: HashMap<String, AgentTask>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            settings: ApiSettings {
                openai_key: None,
                anthropic_key: None,
                gemini_key: None,
            },
            tasks: HashMap::new(),
        }
    }
}

pub type SharedState = Arc<Mutex<AppState>>;
