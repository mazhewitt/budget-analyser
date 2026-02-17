## Context

Phase 1 delivered a SQLite database (`data/budget.db`) with 7,300+ classified transactions and a Jupyter notebook proving the analysis patterns. The recipe-vault project provides a working reference implementation of the exact architecture we need: Axum server, Claude API agent loop, SSE streaming, vanilla JS chat frontend. We adapt that pattern, stripping multi-tenancy, auth, MCP, and image handling, and adding chart artifact rendering.

The database is read-only for this change. The Python import pipeline continues to own data ingestion and classification. This Rust application is a query/analysis layer on top.

## Goals / Non-Goals

**Goals:**
- A working chat where the user can ask spending questions and get narrative answers with inline charts
- Agent loop that reliably selects and executes analysis tools
- Streaming responses so the user sees text arrive in real-time
- Chart artifacts rendered inline in the conversation
- Session management so multi-turn conversations work (e.g. "now break that down by merchant")

**Non-Goals:**
- No authentication (single user, localhost)
- No data import or classification (Python pipeline handles this)
- No write operations to the database (recategorise is a separate change)
- No MCP server — tools are native Rust functions
- No mobile-optimised UI (desktop browser is sufficient)
- No persistent conversation storage (in-memory sessions, lost on restart)

## Decisions

### 1. Adapt recipe-vault architecture directly

**Decision:** Port the server structure, agent loop, SSE streaming, and frontend chat from recipe-vault, stripping what we don't need.

**Rationale:** The recipe-vault architecture is proven, maintained by the same developer, and solves exactly the same problem (LLM + tools + chat UI). Building from scratch would duplicate effort. The simplifications (no auth, no MCP, no images) make this a subset of recipe-vault's complexity.

**What we take:**
- `src/ai/llm.rs` — LLM provider (Anthropic client, message types, tool definitions)
- `src/ai/client.rs` — Agent loop (chat → LLM → tool execution → iterate)
- `src/chat/` — Chat endpoint, session store, SSE streaming
- `static/chat.js` — Message rendering, SSE stream parsing
- `static/app.js` — Application coordinator

**What we drop:**
- Auth middleware, family multi-tenancy, API key management
- MCP server/client, JSON-RPC protocol
- Image upload/vision, photo management
- Recipe-specific models, CRUD endpoints

### 2. Tools as native Rust functions, not MCP

**Decision:** Analysis tools are Rust functions that take parameters and return structured results. No MCP process spawning.

**Rationale:** We have four simple tools that query a single SQLite database. MCP adds process management, JSON-RPC, and a separate binary — unnecessary complexity for this scope. If we later want to share tools across projects, we can extract to MCP then.

**Alternatives considered:**
- MCP server (like recipe-vault) — overhead not justified for 4 read-only tools on one database
- HTTP API endpoints called by agent — adds network round-trip for no benefit when tools run in-process

### 3. SQLite via sqlx, queried directly (no Polars)

**Decision:** Tools execute SQL queries against SQLite using sqlx. No in-memory DataFrame layer.

**Rationale:** The original PROJECT-description.md planned a Polars layer in Rust. However, our queries are straightforward aggregations (GROUP BY category, GROUP BY month, etc.) that SQLite handles natively. Adding polars-rs would increase compile times and binary size for no analytical benefit. The Jupyter notebook uses Polars because Python's SQLite ergonomics are weaker — Rust's sqlx is already excellent for this.

### 4. Chart artifacts via SSE events, rendered with Frappe Charts

**Decision:** Tools return a chart specification alongside their text summary. The server emits this as a `chart_artifact` SSE event. The frontend renders it using Frappe Charts.

**Rationale:** This mirrors recipe-vault's `recipe_artifact` pattern — the LLM doesn't know about the rendering library, it just calls a tool. The tool decides what chart type fits the data. The frontend has a simple mapping from chart spec → Frappe Charts config.

**Chart spec format:**
```json
{
  "type": "bar" | "bar_h" | "pie" | "grouped_bar",
  "title": "Spending by Category",
  "data": {
    "labels": ["Groceries", "Dining", ...],
    "datasets": [
      { "name": "CHF", "values": [1200, 800, ...] }
    ]
  },
  "height": 300
}
```

This maps almost 1:1 to Frappe Charts' constructor format, minimising the translation layer.

### 5. System prompt with schema and tool awareness

**Decision:** The system prompt includes the category schema, a summary of the data model (date range, transaction count, available categories), and instructions for when to use each tool.

**Rationale:** Claude needs context about what's in the database to ask good follow-up questions and provide useful analysis. The category list is small enough to include in full. A data summary (loaded at startup) gives the LLM awareness of the date range and volume without querying on every turn.

### 6. Cargo workspace with single binary

**Decision:** The project is a single Rust binary (not a workspace with multiple crates). Source is organised as modules: `ai/`, `chat/`, `tools/`, `db/`.

**Rationale:** There's no need for crate separation yet. When we later extract reusable components (the agent loop, LLM provider), we can split into a workspace then. Starting with modules keeps build times fast and dependency management simple.

## Risks / Trade-offs

- **Claude API cost** — Each conversation turn costs API tokens. Multi-turn conversations with large tool results could get expensive. → Mitigation: Keep tool result text concise (summaries, not raw data dumps). Chart data goes to the frontend, not back into the LLM context.

- **Tool result size** — A merchant breakdown with 50+ merchants would bloat the LLM context. → Mitigation: Tools default to top-N results (e.g. top 15 merchants) with an "Other" bucket, matching the notebook patterns.

- **Session loss on restart** — In-memory sessions mean conversations are lost when the server restarts. → Acceptable for a local, single-user tool. Persistent sessions can be added later if needed.

- **Database path coupling** — The server assumes `data/budget.db` exists at a relative path. → Mitigation: Make it configurable via environment variable with a sensible default.

- **No chart interactivity feedback** — The LLM can't see what the user sees in the chart (e.g. hovering over a bar). The conversation is text-only. → Acceptable limitation. The user can ask follow-up questions verbally ("what's that big spike in March?").
