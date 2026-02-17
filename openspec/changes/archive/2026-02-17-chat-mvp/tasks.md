## 1. Project Scaffold

- [x] 1.1 Initialise Rust project with Cargo.toml (axum, tokio, sqlx, reqwest, serde, serde_json, tower-http, uuid, tracing, tracing-subscriber)
- [x] 1.2 Create module structure: `src/main.rs`, `src/config.rs`, `src/ai/`, `src/chat/`, `src/tools/`, `src/db/`
- [x] 1.3 Create `static/` directory with `chat.html` placeholder
- [x] 1.4 Implement `Config` struct reading `ANTHROPIC_API_KEY`, `BIND_ADDRESS`, `DATABASE_URL` from env with defaults
- [x] 1.5 Stand up Axum server: serve static files from `static/`, bind to configured address, verify it starts and serves the HTML page

## 2. Database Layer

- [x] 2.1 Create `src/db/mod.rs` with SQLite connection pool setup via sqlx
- [x] 2.2 Implement startup data summary query: date range, total transactions, list of categories with counts — stored in a `DataSummary` struct for the system prompt

## 3. LLM Provider

- [x] 3.1 Create `src/ai/llm.rs` with `LlmProvider` struct: Anthropic HTTP client, message types (`Message`, `ContentBlock`, `ToolCall`, `ToolResult`), `ToolDefinition` struct
- [x] 3.2 Implement `LlmProvider::complete()`: sends messages + tools + system prompt to Claude API, parses response into text and/or tool use blocks
- [x] 3.3 Handle streaming from Claude API — collect streamed response chunks and emit them for SSE forwarding

## 4. Analysis Tools

- [x] 4.1 Create `src/tools/mod.rs` with `ToolRegistry` that maps tool names to handler functions and provides `ToolDefinition` list for the LLM
- [x] 4.2 Implement `spending_by_category` tool: SQL query with optional year/month filters, returns summary text + horizontal bar chart spec
- [x] 4.3 Implement `monthly_trend` tool: SQL query with optional category/year filters, returns summary text + vertical bar chart spec
- [x] 4.4 Implement `merchant_breakdown` tool: SQL query with required category and optional top_n, groups long tail as "Other", returns summary text + bar + pie chart specs
- [x] 4.5 Implement `income_vs_spending` tool: SQL query with optional year filter, returns summary text + grouped bar chart spec
- [x] 4.6 Define the chart specification structs (`ChartSpec`, `ChartData`, `Dataset`) matching the chart-protocol spec

## 5. Agent Loop

- [x] 5.1 Create `src/ai/agent.rs` with `Agent` struct holding the LlmProvider, ToolRegistry, and system prompt
- [x] 5.2 Implement `Agent::chat()`: the core loop — send to LLM, check for tool calls, execute tools, feed results back, repeat (max 10 iterations)
- [x] 5.3 Implement tool result separation: summary text goes back to LLM, chart specs are collected separately for SSE emission
- [x] 5.4 Build the system prompt: category schema, data summary, tool usage guidance

## 6. Chat Endpoint & SSE Streaming

- [x] 6.1 Create `src/chat/sessions.rs` with in-memory `SessionStore`: HashMap of conversation_id → message history, with 2-hour TTL eviction
- [x] 6.2 Create `src/chat/handler.rs` with `POST /api/chat` endpoint: parse request, get/create session, run agent, stream SSE response
- [x] 6.3 Implement SSE event emission: `chunk` (text), `tool_use` (tool indicator), `chart_artifact` (chart spec), `done` (completion), `error`
- [x] 6.4 Implement `POST /api/chat/reset` endpoint: delete session by conversation_id
- [x] 6.5 Wire chat routes into the Axum router

## 7. Frontend Chat UI

- [x] 7.1 Create `static/chat.html`: layout with chat message area, input bar, new chat button
- [x] 7.2 Create `static/chat.js`: send message via POST, parse SSE stream, render text chunks, handle tool_use and done events
- [x] 7.3 Create `static/app.js`: application coordinator — initialise chat, manage conversation_id, handle new chat/reset
- [x] 7.4 Add Frappe Charts: include via CDN link in HTML
- [x] 7.5 Implement chart rendering in `chat.js`: on `chart_artifact` event, create a container div and instantiate a Frappe Chart with the spec data
- [x] 7.6 Add markdown rendering for assistant messages (use a lightweight library or simple regex-based renderer)
- [x] 7.7 Style the chat: user/assistant message distinction, auto-scroll, tool use indicators, responsive layout
- [x] 7.8 Add `static/styles.css` with chat styling

## 8. Integration & Smoke Test

- [x] 8.1 End-to-end test: start server, open browser, ask "How much do I spend by category?", verify text response streams and chart renders
- [x] 8.2 Multi-turn test: ask a follow-up like "Break down groceries by merchant", verify conversation context is maintained
- [x] 8.3 Verify all four tools produce correct chart artifacts
