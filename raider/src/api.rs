use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

use crate::state::{SharedState, ApiSettings, TaskStatus, AgentTask};
use crate::llm::{ModelConfig, create_provider};
use crate::agent::AgentEngine;
use crate::fs_ops::FileOps;
use crate::git_ops::GitContext;

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/settings", get(get_settings).post(update_settings))
        .route("/api/agents", post(start_agent))
        .route("/api/agents/{id}", get(get_agent_status))
        .with_state(state)
}

async fn get_settings(State(state): State<SharedState>) -> impl IntoResponse {
    let settings = state.lock().await.settings.clone();
    Json(settings)
}

async fn update_settings(
    State(state): State<SharedState>,
    Json(payload): Json<ApiSettings>,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    state.settings = payload.clone();
    (StatusCode::OK, Json(payload))
}

#[derive(Deserialize)]
pub struct StartAgentRequest {
    pub target_dir: String,
    pub prompt: String,
    pub provider: String,
    pub model_name: String,
    pub target_files: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct StartAgentResponse {
    pub task_id: String,
}

async fn start_agent(
    State(state): State<SharedState>,
    Json(payload): Json<StartAgentRequest>,
) -> impl IntoResponse {
    let task_id = Uuid::new_v4().to_string();

    let settings = state.lock().await.settings.clone();
    let api_key = match payload.provider.as_str() {
        "openai" => settings.openai_key,
        "anthropic" => settings.anthropic_key,
        "gemini" => settings.gemini_key,
        _ => None,
    };

    let model_config = ModelConfig {
        provider: payload.provider.clone(),
        model_name: payload.model_name.clone(),
        api_key,
    };

    let task = AgentTask {
        id: task_id.clone(),
        target_dir: payload.target_dir.clone(),
        prompt: payload.prompt.clone(),
        model: model_config.clone(),
        status: TaskStatus::Pending,
        logs: vec!["Task created.".to_string()],
    };

    state.lock().await.tasks.insert(task_id.clone(), task);

    // Spawn agent task in background
    let state_clone = state.clone();
    let target_dir = payload.target_dir.clone();
    let prompt = payload.prompt.clone();
    let target_files_req = payload.target_files.clone();
    let task_id_clone = task_id.clone();

    tokio::spawn(async move {
        // Update status: Reading Files
        {
            let mut st = state_clone.lock().await;
            if let Some(t) = st.tasks.get_mut(&task_id_clone) {
                t.status = TaskStatus::ReadingFiles;
                t.logs.push("Scanning directory/files...".to_string());
            }
        }

        let target_path = PathBuf::from(&target_dir);
        let files = match target_files_req {
            Some(f) if !f.is_empty() => f.into_iter().map(|p| target_path.join(p)).collect::<Vec<_>>(),
            _ => {
                match FileOps::list_files(&target_path) {
                    Ok(f) => f,
                    Err(e) => {
                        update_status_error(&state_clone, &task_id_clone, format!("Failed to list files: {}", e)).await;
                        return;
                    }
                }
            }
        };

        // Update status: Thinking
        {
            let mut st = state_clone.lock().await;
            if let Some(t) = st.tasks.get_mut(&task_id_clone) {
                t.status = TaskStatus::Thinking;
                t.logs.push(format!("Sending {} files context to LLM...", files.len()));
            }
        }

        let provider = match create_provider(&model_config) {
            Ok(p) => p,
            Err(e) => {
                update_status_error(&state_clone, &task_id_clone, format!("Provider error: {}", e)).await;
                return;
            }
        };

        // Open git repo if possible
        let git_ctx = GitContext::open(&target_dir).ok();
        let branch_name = format!("raider-task-{}", &task_id_clone[0..8]);

        if let Some(ref git) = git_ctx {
            if let Err(e) = git.create_and_checkout_branch(&branch_name) {
                tracing::warn!("Failed to create git branch: {}", e);
            } else {
                let mut st = state_clone.lock().await;
                if let Some(t) = st.tasks.get_mut(&task_id_clone) {
                    t.logs.push(format!("Created git branch: {}", branch_name));
                }
            }
        }

        let engine = AgentEngine::new(provider, target_dir);

        match engine.run_task(&prompt, &files).await {
            Ok(_) => {
                // Update status: Committing
                {
                    let mut st = state_clone.lock().await;
                    if let Some(t) = st.tasks.get_mut(&task_id_clone) {
                        t.status = TaskStatus::Committing;
                        t.logs.push("Committing changes...".to_string());
                    }
                }

                if let Some(ref git) = git_ctx {
                    match git.commit_all(&prompt) {
                        Ok(oid) => {
                            let mut st = state_clone.lock().await;
                            if let Some(t) = st.tasks.get_mut(&task_id_clone) {
                                t.logs.push(format!("Committed changes with OID: {}", oid));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to commit changes: {}", e);
                        }
                    }
                }

                let mut st = state_clone.lock().await;
                if let Some(t) = st.tasks.get_mut(&task_id_clone) {
                    t.status = TaskStatus::Done;
                    t.logs.push("Task completed successfully!".to_string());
                }
            }
            Err(e) => {
                update_status_error(&state_clone, &task_id_clone, format!("Agent error: {}", e)).await;
            }
        }
    });

    (StatusCode::OK, Json(StartAgentResponse { task_id }))
}

async fn update_status_error(state: &SharedState, task_id: &str, err_msg: String) {
    let mut st = state.lock().await;
    if let Some(t) = st.tasks.get_mut(task_id) {
        t.status = TaskStatus::Error(err_msg.clone());
        t.logs.push(err_msg);
    }
}

async fn get_agent_status(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let state = state.lock().await;
    if let Some(task) = state.tasks.get(&id) {
        (StatusCode::OK, Json(task.clone())).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Task not found").into_response()
    }
}
