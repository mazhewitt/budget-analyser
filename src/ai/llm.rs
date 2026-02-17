use serde::{Deserialize, Serialize};
use futures_util::StreamExt;

#[derive(Debug, Clone)]
pub struct LlmProvider {
    pub client: reqwest::Client,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
}

impl LlmProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            max_tokens: 1024,
        }
    }

    pub async fn complete(
        &self,
        system: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<LlmCompletion, LlmError> {
        let request = ClaudeMessageRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            system: system.to_string(),
            messages: messages.to_vec(),
            tools: tools.to_vec(),
            stream: false,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(LlmError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api { status, body });
        }

        let reply: ClaudeMessageResponse = response.json().await.map_err(LlmError::Http)?;

        Ok(LlmCompletion {
            content: reply.content,
            stop_reason: reply.stop_reason,
        })
    }

    pub async fn complete_stream(
        &self,
        system: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<StreamedCompletion, LlmError> {
        let request = ClaudeMessageRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            system: system.to_string(),
            messages: messages.to_vec(),
            tools: tools.to_vec(),
            stream: true,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(LlmError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api { status, body });
        }

        let mut content_blocks: Vec<ContentBlock> = Vec::new();
        let mut chunks: Vec<StreamChunk> = Vec::new();
        let mut buffer = String::new();
        let mut current_tool: Option<StreamingToolUse> = None;

        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let bytes = item.map_err(LlmError::Http)?;
            let part = String::from_utf8_lossy(&bytes);
            buffer.push_str(&part);

            while let Some(idx) = buffer.find("\n\n") {
                let event = buffer[..idx].to_string();
                buffer = buffer[idx + 2..].to_string();

                for line in event.lines() {
                    let Some(data) = line.strip_prefix("data: ") else { continue };
                    if data.trim() == "[DONE]" {
                        continue;
                    }

                    let value: serde_json::Value = match serde_json::from_str(data) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let event_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    match event_type {
                        "content_block_start" => {
                            if let Some(block) = value.get("content_block") {
                                if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                                    if block_type == "tool_use" {
                                        let id = block.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        current_tool = Some(StreamingToolUse {
                                            id,
                                            name,
                                            input: String::new(),
                                        });
                                    }
                                }
                            }
                        }
                        "content_block_delta" => {
                            if let Some(delta) = value.get("delta") {
                                let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                if delta_type == "text_delta" {
                                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                        chunks.push(StreamChunk::Text(text.to_string()));
                                        match content_blocks.last_mut() {
                                            Some(ContentBlock::Text { text: existing }) => existing.push_str(text),
                                            _ => content_blocks.push(ContentBlock::Text { text: text.to_string() }),
                                        }
                                    }
                                } else if delta_type == "input_json_delta" {
                                    if let Some(partial) = delta.get("partial_json").and_then(|v| v.as_str()) {
                                        if let Some(tool) = current_tool.as_mut() {
                                            tool.input.push_str(partial);
                                        }
                                    }
                                }
                            }
                        }
                        "content_block_stop" => {
                            if let Some(tool) = current_tool.take() {
                                let input_value = serde_json::from_str(&tool.input).unwrap_or(serde_json::Value::Null);
                                let call = ToolCall {
                                    id: tool.id,
                                    name: tool.name,
                                    input: input_value,
                                };
                                content_blocks.push(ContentBlock::ToolUse(call.clone()));
                                chunks.push(StreamChunk::ToolUse(call));
                            }
                        }
                        "message_stop" => {
                            let stop_reason = value.get("stop_reason").and_then(|v| v.as_str()).map(|s| s.to_string());
                            chunks.push(StreamChunk::Done { stop_reason: stop_reason.clone() });
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(StreamedCompletion {
            content: content_blocks,
            chunks,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    ToolUse(ToolCall),
    ToolResult(ToolResult),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct ClaudeMessageRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
    tools: Vec<ToolDefinition>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ClaudeMessageResponse {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LlmCompletion {
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StreamedCompletion {
    pub content: Vec<ContentBlock>,
    pub chunks: Vec<StreamChunk>,
}

#[derive(Debug, Clone)]
pub enum StreamChunk {
    Text(String),
    ToolUse(ToolCall),
    Done { stop_reason: Option<String> },
}

#[derive(Debug, Clone)]
struct StreamingToolUse {
    id: String,
    name: String,
    input: String,
}

#[derive(Debug)]
pub enum LlmError {
    Http(reqwest::Error),
    Api { status: reqwest::StatusCode, body: String },
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Http(err) => write!(f, "HTTP error: {}", err),
            LlmError::Api { status, body } => write!(f, "API error {}: {}", status, body),
        }
    }
}
