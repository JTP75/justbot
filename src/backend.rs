use axum::{Router, response::IntoResponse, routing};
use tokio::signal;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let _ = env_logger::builder().try_init();

    let app = Router::new()
        .route("/health", routing::get(health_check))
        .route("/mcp", routing::get(mcp_handler))
        .route("/embedding", routing::get(embedding_handler))
        .route("/qdrant", routing::get(qdrant_handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8081")
        .await.map_err(|e| format!("Failed to create listener: {}", e))?;

    log::info!("Backend hosted on http://127.0.0.1:8081");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await.map_err(|e| format!("Server failed: {}", e))?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn mcp_handler() -> impl IntoResponse {
    "MCP service"
}

async fn embedding_handler() -> impl IntoResponse {
    "Embedding service"
}

async fn qdrant_handler() -> impl IntoResponse {
    "Qdrant service"
}

async fn shutdown_signal() {
    signal::ctrl_c().await
        .expect("failed to install CTRL+C signal handler");
    log::info!("\nShutdown signal received");
}