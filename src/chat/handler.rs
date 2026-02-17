use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::ai::agent::{Agent, AgentEvent};
use crate::chat::sessions::SessionStore;

#[derive(Clone)]
pub struct ChatState {
    pub agent: Agent,
    pub sessions: SessionStore,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
pub struct SseChunk {
    pub text: String,
}

#[derive(Serialize)]
pub struct SseToolUse {
    pub tool: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct SseChartArtifact {
    #[serde(rename = "type")]
    pub chart_type: String,
    pub title: String,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

#[derive(Serialize)]
pub struct SseDone {
    pub conversation_id: String,
    pub stop_reason: Option<String>,
}

#[derive(Serialize)]
pub struct SseError {
    pub message: String,
}

#[derive(Deserialize)]
pub struct ResetRequest {
    pub conversation_id: String,
}

pub async fn chat(
    State(state): State<ChatState>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl futures_util::stream::Stream<Item = Result<Event, std::fmt::Error>>>, StatusCode> {
    let msg = req.message.clone();
    let conv_id = req.conversation_id.clone();
    let agent = state.agent.clone();
    let sessions = state.sessions.clone();

    let stream = async_stream::stream! {
        let (conversation_id, mut history) = sessions.get_or_create(conv_id.as_deref()).await;
        info!(conversation_id = %conversation_id, message = %msg, history_len = history.len(), "chat request");

        match agent.chat(&mut history, &msg).await {
            Ok((reply, events)) => {
                info!(conversation_id = %conversation_id, tools = ?reply.tools_used, events = events.len(), "chat complete");
                sessions.save_history(&conversation_id, history).await;

                // Emit agent events in order (tool_use + chart_artifact)
                for event in events {
                    match event {
                        AgentEvent::ToolRunning { tool } => {
                            let sse = Event::default()
                                .event("tool_use")
                                .json_data(SseToolUse { tool, status: "running".to_string() });
                            if let Ok(evt) = sse {
                                yield Ok(evt);
                            }
                        }
                        AgentEvent::ToolCompleted { tool } => {
                            let sse = Event::default()
                                .event("tool_use")
                                .json_data(SseToolUse { tool, status: "completed".to_string() });
                            if let Ok(evt) = sse {
                                yield Ok(evt);
                            }
                        }
                        AgentEvent::ChartArtifact(chart) => {
                            let artifact = SseChartArtifact {
                                chart_type: chart.chart_type,
                                title: chart.title,
                                data: serde_json::to_value(&chart.data).unwrap_or(serde_json::Value::Null),
                                height: chart.height,
                            };
                            let sse = Event::default()
                                .event("chart_artifact")
                                .json_data(artifact);
                            if let Ok(evt) = sse {
                                yield Ok(evt);
                            }
                        }
                    }
                }

                // Stream text response
                if !reply.text.is_empty() {
                    for chunk in reply.text.split_whitespace() {
                        let sse = Event::default()
                            .event("chunk")
                            .json_data(SseChunk {
                                text: format!("{} ", chunk),
                            });
                        if let Ok(evt) = sse {
                            yield Ok(evt);
                        }
                    }
                }

                let reason = if reply.incomplete {
                    Some("max_iterations".to_string())
                } else {
                    Some("end_turn".to_string())
                };
                let done_event = Event::default()
                    .event("done")
                    .json_data(SseDone {
                        conversation_id: conversation_id.clone(),
                        stop_reason: reason
                    });
                if let Ok(evt) = done_event {
                    yield Ok(evt);
                }
            }
            Err(err) => {
                let msg = format!("{:?}", err);
                error!(conversation_id = %conversation_id, error = %msg, "agent error");
                let err_event = Event::default()
                    .event("error")
                    .json_data(SseError { message: msg });
                if let Ok(evt) = err_event {
                    yield Ok(evt);
                }
            }
        }
    };

    Ok(Sse::new(stream))
}

pub async fn reset(
    State(state): State<ChatState>,
    Json(req): Json<ResetRequest>,
) -> StatusCode {
    state.sessions.delete(&req.conversation_id).await;
    StatusCode::OK
}

pub fn router(state: ChatState) -> Router {
    Router::new()
        .route("/api/chat", post(chat))
        .route("/api/chat/reset", post(reset))
        .with_state(state)
}
