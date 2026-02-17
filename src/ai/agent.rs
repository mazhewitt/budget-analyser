use crate::ai::llm::{ContentBlock, LlmCompletion, LlmError, LlmProvider, Message, ToolCall, ToolResult};
use crate::db::{CategoryInfo, DataSummary};
use crate::tools::{ChartSpec, ToolError, ToolRegistry};
use sqlx::SqlitePool;
use tracing::{info, warn, debug};

#[derive(Clone)]
pub struct Agent {
    llm: LlmProvider,
    tools: ToolRegistry,
    system_prompt: String,
    pool: SqlitePool,
}

pub struct AgentReply {
    pub text: String,
    pub charts: Vec<ChartSpec>,
    pub tools_used: Vec<String>,
    pub incomplete: bool,
}

/// Events emitted during the agent loop, in order.
#[derive(Debug)]
pub enum AgentEvent {
    ToolRunning { tool: String },
    ToolCompleted { tool: String },
    ChartArtifact(ChartSpec),
}

#[derive(Debug)]
pub enum AgentError {
    Llm(LlmError),
    Tool(ToolError),
}

impl Agent {
    pub fn new(llm: LlmProvider, tools: ToolRegistry, system_prompt: String, pool: SqlitePool) -> Self {
        Self {
            llm,
            tools,
            system_prompt,
            pool,
        }
    }

    pub async fn chat(
        &self,
        history: &mut Vec<Message>,
        user_input: &str,
    ) -> Result<(AgentReply, Vec<AgentEvent>), AgentError> {
        history.push(Message {
            role: "user".to_string(),
            content: vec![ContentBlock::Text { text: user_input.to_string() }],
        });

        let mut charts: Vec<ChartSpec> = Vec::new();
        let mut tools_used: Vec<String> = Vec::new();
        let mut events: Vec<AgentEvent> = Vec::new();
        let mut final_text = String::new();
        let mut incomplete = false;

        for iteration in 0..10 {
            debug!(iteration, "llm call");
            let completion = self
                .llm
                .complete(&self.system_prompt, history, self.tools.definitions())
                .await
                .map_err(AgentError::Llm)?;

            let (assistant_message, tool_calls, text_chunk) = unpack_completion(&completion);
            if !assistant_message.content.is_empty() {
                history.push(assistant_message);
            }

            if !text_chunk.is_empty() {
                debug!(text_len = text_chunk.len(), "llm text response");
                final_text.push_str(&text_chunk);
            }

            if tool_calls.is_empty() {
                info!(iteration, "agent done (no more tool calls)");
                break;
            }

            for tool_call in &tool_calls {
                info!(tool = %tool_call.name, input = %tool_call.input, "tool call");
                events.push(AgentEvent::ToolRunning { tool: tool_call.name.clone() });

                let result = match self
                    .tools
                    .run(&self.pool, &tool_call.name, tool_call.input.clone())
                    .await
                {
                    Ok(output) => {
                        info!(tool = %tool_call.name, summary_len = output.summary.len(), charts = output.charts.len(), "tool ok");
                        for chart in output.charts {
                            events.push(AgentEvent::ChartArtifact(chart));
                        }
                        tools_used.push(tool_call.name.clone());
                        events.push(AgentEvent::ToolCompleted { tool: tool_call.name.clone() });
                        ToolResult {
                            tool_use_id: tool_call.id.clone(),
                            content: output.summary,
                            is_error: None,
                        }
                    }
                    Err(err) => {
                        warn!(tool = %tool_call.name, error = ?err, "tool error");
                        events.push(AgentEvent::ToolCompleted { tool: tool_call.name.clone() });
                        ToolResult {
                            tool_use_id: tool_call.id.clone(),
                            content: format!("Tool error: {:?}", err),
                            is_error: Some(true),
                        }
                    },
                };

                history.push(Message {
                    role: "user".to_string(),
                    content: vec![ContentBlock::ToolResult(result)],
                });
            }

            if iteration == 9 {
                incomplete = true;
            }
        }

        // Collect charts from events for the reply
        for event in &events {
            if let AgentEvent::ChartArtifact(chart) = event {
                charts.push(chart.clone());
            }
        }

        Ok((
            AgentReply {
                text: final_text,
                charts,
                tools_used,
                incomplete,
            },
            events,
        ))
    }
}

pub fn build_system_prompt(summary: &DataSummary, categories: &[CategoryInfo]) -> String {
    let mut category_lines = String::new();
    for cat in categories {
        category_lines.push_str(&format!("- {}: {}\n", cat.name, cat.description));
    }

    let mut category_counts = String::new();
    for entry in &summary.categories {
        category_counts.push_str(&format!("- {} ({} tx)\n", entry.name, entry.count));
    }

    let date_range = match (&summary.min_date, &summary.max_date) {
        (Some(min), Some(max)) => format!("{} to {}", min, max),
        _ => "unknown range".to_string(),
    };

    format!(
        "You are a budget analysis assistant. Use the provided tools to answer questions about spending.\n\n
tDATA SUMMARY\n- Date range: {}\n- Total transactions: {}\n- Categories and counts:\n{}\n\nCATEGORY SCHEMA\n{}\nTOOLS\n- spending_by_category: totals by category with optional year/month filters\n- monthly_trend: monthly spending totals with optional category/year filters\n- merchant_breakdown: top merchants within a category\n- income_vs_spending: monthly income vs spending, optional year filter\n\nGuidance: keep summaries concise, and use tools for quantitative questions.",
        date_range,
        summary.total_transactions,
        category_counts,
        category_lines
    )
}

fn unpack_completion(completion: &LlmCompletion) -> (Message, Vec<ToolCall>, String) {
    let mut tool_calls = Vec::new();
    let mut text = String::new();

    for block in &completion.content {
        match block {
            ContentBlock::Text { text: chunk } => {
                text.push_str(chunk);
            }
            ContentBlock::ToolUse(tool) => tool_calls.push(tool.clone()),
            ContentBlock::ToolResult(_) => {}
        }
    }

    let message = Message {
        role: "assistant".to_string(),
        content: completion.content.clone(),
    };

    (message, tool_calls, text)
}
