// ! HTTP API server for Web UI

use axum::{
    Router,
    extract::State,
    http::{Method, StatusCode, header},
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tower_http::cors::{Any, CorsLayer};

/// Commands sent from HTTP handlers to the P2P swarm
#[derive(Debug)]
pub enum SwarmCommand {
    Ask {
        prompt: String,
        responder: oneshot::Sender<Result<String, String>>,
    },
}

/// HTTP request payload for /api/ask
#[derive(Debug, Deserialize)]
pub struct AskRequest {
    pub prompt: String,
}

/// HTTP response payload for /api/ask
#[derive(Debug, Serialize)]
pub struct AskResponse {
    pub answer: String,
}

/// HTTP response for errors
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Shared state for HTTP handlers
#[derive(Clone)]
pub struct AppState {
    pub command_tx: mpsc::Sender<SwarmCommand>,
}

/// Start the HTTP API server
pub async fn start_server(command_tx: mpsc::Sender<SwarmCommand>) -> anyhow::Result<()> {
    let state = AppState { command_tx };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/ask", post(handle_ask))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("🌐 HTTP API listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Health check endpoint
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Handle /api/ask endpoint
async fn handle_ask(
    State(state): State<AppState>,
    Json(payload): Json<AskRequest>,
) -> Result<Json<AskResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Create a oneshot channel to receive the answer
    let (resp_tx, resp_rx) = oneshot::channel();

    // Send command to P2P swarm
    state
        .command_tx
        .send(SwarmCommand::Ask {
            prompt: payload.prompt,
            responder: resp_tx,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to send command: {}", e),
                }),
            )
        })?;

    // Wait for response from P2P swarm (with timeout)
    let answer = tokio::time::timeout(std::time::Duration::from_secs(120), resp_rx)
        .await
        .map_err(|_| {
            (
                StatusCode::REQUEST_TIMEOUT,
                Json(ErrorResponse {
                    error: "Request timeout".to_string(),
                }),
            )
        })?
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Channel closed".to_string(),
                }),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    Ok(Json(AskResponse { answer }))
}
