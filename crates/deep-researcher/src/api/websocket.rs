use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::api::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct WsIncomingMessage {
    #[allow(dead_code)]
    pub message_type: Option<String>,
    pub content: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WsOutgoingMessage {
    pub message_type: String, // "system_response", "intermediate_step", "error"
    pub content: String,
    pub is_final: bool,
    pub status: String,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    info!("New WebSocket client connected for research interaction");

    let coordinator = Arc::clone(&state.coordinator);
    let session_id = uuid::Uuid::new_v4().to_string();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let text_str = text.as_str();
                debug!("Received WS message: {}", text_str);

                let query = if let Ok(parsed) = serde_json::from_str::<WsIncomingMessage>(text_str) {
                    parsed.query.or(parsed.content).unwrap_or_else(|| text_str.to_string())
                } else {
                    text_str.to_string()
                };

                let q = query.trim().to_string();
                if q.is_empty() {
                    continue;
                }

                // Send intermediate step: "Analyzing query..."
                let step_msg = WsOutgoingMessage {
                    message_type: "intermediate_step".to_string(),
                    content: "Analyzing research intent and gathering sources...".to_string(),
                    is_final: false,
                    status: "in_progress".to_string(),
                };
                let _ = sender.send(Message::Text(serde_json::to_string(&step_msg).unwrap().into())).await;

                // Run workflow
                match coordinator.run_workflow(&session_id, &q).await {
                    Ok(report) => {
                        let final_msg = WsOutgoingMessage {
                            message_type: "system_response".to_string(),
                            content: report,
                            is_final: true,
                            status: "completed".to_string(),
                        };
                        let _ = sender.send(Message::Text(serde_json::to_string(&final_msg).unwrap().into())).await;
                    }
                    Err(err) => {
                        let err_msg = WsOutgoingMessage {
                            message_type: "error".to_string(),
                            content: format!("Research failed: {}", err),
                            is_final: true,
                            status: "error".to_string(),
                        };
                        let _ = sender.send(Message::Text(serde_json::to_string(&err_msg).unwrap().into())).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket client closed connection");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
