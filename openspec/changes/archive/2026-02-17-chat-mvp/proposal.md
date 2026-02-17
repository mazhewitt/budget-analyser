## Why

Spending analysis currently requires working in a Jupyter notebook with an engineer driving the queries. Phase 1 delivered clean, classified data in SQLite and proved the analysis patterns (category breakdowns, merchant deep dives, monthly trends, recategorisation). Now we need to make that analysis accessible through a conversational interface where an LLM agent picks the right queries, renders charts, and explains the results — replacing the notebook with a chat.

## What Changes

- New Rust/Axum web server serving a browser-based chat interface
- LLM agent (Claude API) with tool calling for budget analysis
- Four read-only analysis tools: `spending_by_category`, `monthly_trend`, `merchant_breakdown`, `income_vs_spending`
- SSE streaming protocol for real-time text, tool use indicators, and chart artifacts
- Vanilla JS frontend with Frappe Charts for inline chart rendering
- In-memory conversation session management
- System prompt tailored to budget analysis with awareness of the category schema and data model

## Capabilities

### New Capabilities
- `chat-server`: Axum HTTP server serving static files and the chat API endpoint, with SSE streaming
- `llm-agent`: Claude API provider, agent loop (message → LLM → tool calls → results → LLM), conversation history management
- `analysis-tools`: The four read-only query tools that run against SQLite and return structured data plus chart specifications
- `chat-ui`: Browser chat interface with message streaming, tool use indicators, and Frappe Charts rendering for chart artifacts
- `chart-protocol`: The chart artifact specification — the contract between tools (which produce chart data) and the frontend (which renders it)

### Modified Capabilities
(none — this is a new application layer on top of the existing data pipeline)

## Impact

- **New crate/binary**: A Rust application alongside the existing Python pipeline
- **Dependencies**: axum, tokio, sqlx, reqwest (for Claude API), serde, tower-http (static files)
- **Frontend assets**: HTML, JS, CSS in a `static/` directory, Frappe Charts loaded from CDN or vendored
- **Database**: Read-only access to existing `data/budget.db` — no schema changes
- **External service**: Claude API (requires `ANTHROPIC_API_KEY` environment variable)
- **Architecture reference**: Adapted from recipe-vault (same team, same patterns)
