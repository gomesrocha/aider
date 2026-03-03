pub mod git_ops;
pub mod fs_ops;
pub mod llm;
pub mod agent;
pub mod state;
pub mod api;

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Raider initializing...");

    let state = Arc::new(Mutex::new(AppState::new()));

    // Create the router containing the API routes and static file serving
    let api_routes = api::create_router(state.clone());

    // Serve frontend static files
    let frontend = tower_http::services::ServeDir::new("static")
        .fallback(tower_http::services::ServeFile::new("static/index.html"));

    let app = axum::Router::new()
        .fallback_service(frontend)
        .merge(api_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
