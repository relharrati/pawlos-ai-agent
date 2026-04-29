use std::sync::Arc;
use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message as WsMessage}},
    routing::{get, post},
    Router, Json, response::IntoResponse,
};
use tower_http::cors::CorsLayer;
use tower_http::fs::ServeDir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use pawlos_core::Config;
use provider::registry::ProviderRegistry;
use crate::session::SessionManager;
use crate::turn::run_turn;

#[derive(Clone)]
pub struct AppState {
    pub session_mgr: Arc<SessionManager>,
    pub provider: Arc<ProviderRegistry>,
    pub model: String,
    pub agent_name: String,
}

#[derive(Deserialize)]
struct ChatPayload {
    message: String,
    session_id: Option<String>,
    model: Option<String>,
}

#[derive(Serialize)]
struct ChatReply {
    session_id: String,
    content: String,
}

pub struct WebServer {
    pub host: String,
    pub port: u16,
}

impl WebServer {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            host: cfg.server.host.clone(),
            port: cfg.server.port,
        }
    }

    pub async fn run(self, state: AppState) -> Result<()> {
        let web_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("web")))
            .unwrap_or_else(|| std::path::PathBuf::from("web"));

        let app = Router::new()
            .route("/api/chat", post(chat_handler))
            .route("/ws", get(ws_handler))
            .nest_service("/", ServeDir::new(&web_dir))
            .layer(CorsLayer::permissive())
            .with_state(Arc::new(state));

        let addr = format!("{}:{}", self.host, self.port);
        tracing::info!("🌐 Web UI at http://{addr}");

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatPayload>,
) -> impl IntoResponse {
    let session_id = payload
        .session_id
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::new_v4);

    // Ensure session exists
    let _ = state.session_mgr.get_or_create(&state.agent_name).await;

    let model = payload.model.as_deref().unwrap_or(&state.model).to_string();

    match run_turn(
        session_id,
        payload.message,
        &state.agent_name,
        &state.session_mgr,
        &state.provider,
        &model,
        true,
    )
    .await
    {
        Ok((content, _)) => Json(ChatReply {
            session_id: session_id.to_string(),
            content,
        })
        .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
        )
            .into_response(),
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>) {
    let session_id = Uuid::new_v4();
    let _ = state.session_mgr.get_or_create(&state.agent_name).await;

    while let Some(Ok(msg)) = {
        use futures::StreamExt;
        socket.recv().await.map(|r| r.map(|m| m))
    } {
        let text = match msg {
            WsMessage::Text(t) => t,
            WsMessage::Close(_) => break,
            _ => continue,
        };

        let response = match run_turn(
            session_id,
            text.to_string(),
            &state.agent_name,
            &state.session_mgr,
            &state.provider,
            &state.model,
            true,
        )
        .await
        {
            Ok((content, _)) => content,
            Err(e) => format!("Error: {e}"),
        };

        if socket.send(WsMessage::Text(response.into())).await.is_err() {
            break;
        }
    }
}
