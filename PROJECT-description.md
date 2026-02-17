# Budget Analyser

## Overview

A personal financial tracking application that imports transaction history from UBS accounts, classifies cryptic merchant strings using LLMs, and provides an interactive AI chat interface for exploring spending data. Designed for a single user running locally on a Mac Studio.

## Goals

Turn opaque bank statements into a clear, categorised view of spending habits — through an interactive chat where the AI does the heavy lifting and the human steers the analysis.

Specifically:

1. **Import**: Parse UBS CSV exports into a structured format, handling Swiss-specific formatting (apostrophe thousands separators, mixed currency transactions). Detect and skip duplicate imports.
2. **Classify**: Use LLMs to interpret cryptic merchant descriptions and map them to normalised merchant names and spending categories. Build a merchant cache that learns over time so the LLM handles cold starts while repeated merchants resolve instantly.
3. **Analyse**: Ask natural language questions about spending ("How much did I spend on dining this year?", "Break down groceries by store") through an AI chat interface. The AI uses tool calling to query the database, generate charts, and explain the results conversationally.
4. **Recategorise**: Correct misclassifications through the chat interface. The AI updates the database and the corrections improve future analysis.
5. **Store**: Persist everything in SQLite for querying, trend analysis, and long-term history.

## Design Principles

**Manual CSV import.** Switzerland lacks mandated Open Banking APIs. Rather than building brittle scraping against the UBS web interface, we accept a monthly manual CSV export. This is a reliable, stable data source that won't break when UBS updates their frontend.

**AI-first interaction.** The primary interface is a chat. The user asks questions in natural language; the AI picks the right tools, runs queries, renders charts, and explains what it finds. No dashboards, no forms — just a conversation about your money.

**Tool use over SQL generation.** The chat interface exposes a small set of coarse-grained query tools via the Claude API's tool calling. The LLM's job is to pick the right tool and parameters — the actual data work happens in Rust functions querying SQLite directly. This is safer, more reliable, and keeps the tool interface as a clear contract between the AI and the system. A raw SQL tool may be added later as an escape hatch with read-only guardrails.

**Chart artifacts.** When a tool returns data suitable for visualisation, the server emits a `chart_artifact` SSE event alongside the text stream. The browser renders charts inline using Frappe Charts. The LLM doesn't need to know the charting library — it calls a tool, the tool returns structured chart data, and the frontend renders it.

**LLM as cold-start engine, not permanent dependency.** For classification, the LLM bootstraps the merchant cache. Over time, the vast majority of transactions resolve from the cache and the LLM is only invoked for genuinely new merchants. This keeps classification fast and deterministic where it can be.

**Dynamic few-shot learning.** When the user corrects a misclassification, the correction is stored as an example keyed by merchant pattern. These examples are dynamically injected into the LLM prompt, so the system learns from its mistakes without retraining.

**Amount-aware classification.** Monetary values are essential context for accurate categorisation. A 6 CHF transaction at Coop Pronto is a coffee; a 145 CHF transaction at the same merchant is a grocery shop. The LLM receives the full transaction including amount.

## Category Schema

A flat, single-level category list. Kept deliberately simple — over-engineering the taxonomy is a common trap that makes classification harder without adding analytical value.

| Category | Covers |
|---|---|
| Groceries | Supermarkets, food shops, bakeries, butchers |
| Dining | Restaurants, cafes, bars, takeaway, fast food |
| Transport | Public transport, taxis, fuel, parking, car expenses |
| Housing | Rent, mortgage, utilities, electricity, water, heating |
| Insurance | Health insurance, liability, household, car insurance |
| Healthcare | Doctors, dentists, pharmacy, hospital, optician |
| Shopping | Clothing, electronics, furniture, household goods |
| Subscriptions | Streaming, software, newspapers, memberships, phone plan |
| Children | Childcare, school, activities, toys, children's clothing |
| Travel | Hotels, flights, holiday expenses |
| Investments | Pension contributions, ETFs, investment fund purchases |
| Cash | ATM withdrawals |
| Transfers | Transfers between own accounts, savings |
| Income | Salary, refunds, reimbursements |
| Fees | Bank fees, card fees, foreign exchange fees |
| Taxes | Federal, cantonal, and municipal tax payments |
| Other | Anything that doesn't fit above |

## Data Model

Four core tables:

**transactions** — every imported row from UBS, with its categorisation.

- `id`, `date`, `raw_description`, `amount`, `currency`, `merchant_name`, `category`, `source` (llm/manual/cache), `confidence`, `import_batch`, `created_at`

**merchant_cache** — the learned mapping from raw merchant strings to normalised names and categories.

- `raw_key` (normalised lookup key), `merchant_name`, `category`, `confidence`, `source` (llm/manual), `created_at`, `updated_at`

**import_log** — tracks each CSV import to prevent duplicates.

- `id`, `filename`, `row_count`, `imported_at`

**few_shot_examples** — user corrections stored as dynamic prompt examples.

- `id`, `merchant_pattern`, `raw_description`, `correct_merchant`, `correct_category`, `created_at`

## Tech Stack

### Phase 1 (Python — data pipeline)

- **Python** for CSV parsing, LLM classification, and exploratory analysis
- **SQLite** for storage and persistence
- **Polars** for data analysis in Jupyter notebooks
- **Ollama** for local LLM inference (classification)
- **Models:** Qwen3 8B for classification

### Phase 2 (Rust — chat application)

- **Rust** for the backend (web server, agent loop, tool execution)
- **Axum** for the HTTP server and SSE streaming
- **Vanilla JavaScript** for the browser chat UI (no framework, no build step)
- **Frappe Charts** for inline chart rendering in chat
- **SQLite** for storage and persistence (via sqlx)
- **Claude API** (Anthropic) for the chat agent with tool calling
- **Architecture pattern** adapted from [recipe-vault](../recipe-vault): Axum server + agent loop + SSE streaming + vanilla JS frontend

## Analysis Tools

The chat agent has access to these tools. Each tool queries SQLite directly and returns structured data plus optional chart specifications for the frontend to render.

| Tool | Parameters | Returns |
|---|---|---|
| `spending_by_category` | `year?`, `month?` | Table of categories with totals and transaction counts, horizontal bar chart |
| `monthly_trend` | `category?`, `year?` | Monthly time series of spending, vertical bar chart |
| `merchant_breakdown` | `category`, `top_n?` | Top merchants ranked by spend with pie + bar charts |
| `income_vs_spending` | `year?` | Monthly income vs expenses, grouped bar chart |
| `recategorise` | `merchant_name`, `new_category` | Updates transactions and merchant cache, confirmation |

Tools return both a text summary (for the LLM to narrate) and a chart specification (for the frontend to render). The LLM never sees chart rendering details — it just calls the tool and explains the results.

## SSE Protocol

The chat endpoint streams responses using Server-Sent Events, adapted from the recipe-vault pattern:

| Event | Data | Purpose |
|---|---|---|
| `chunk` | `{"text": "..."}` | Streaming text from the LLM |
| `tool_use` | `{"tool": "...", "status": "..."}` | Tool call indicator |
| `chart_artifact` | `{"type": "bar", "title": "...", "data": {...}}` | Chart for Frappe Charts to render |
| `done` | `{"conversation_id": "...", "tools_used": [...]}` | Conversation turn complete |
| `error` | `{"message": "..."}` | Error |

## Implementation Roadmap

---

### Phase 1: Data Pipeline (complete)

Build a reliable data pipeline and get clean, classified data into SQLite.

**What was delivered:**

- Python-based CSV import pipeline with LLM classification via Ollama
- SQLite database (`data/budget.db`) with 7,300+ classified transactions
- Merchant cache providing ~65% cache hit rate, reducing LLM calls over time
- Dynamic few-shot examples from manual corrections
- Jupyter notebook (`analysis.ipynb`) with comprehensive analysis: spending by category, monthly trends, merchant breakdowns, income vs spending, deep dives into Groceries, Shopping, Dining, and Other categories

**What was learned during Phase 1:**

The Jupyter notebook analysis revealed the kinds of questions and interactions that the chat interface needs to support: category breakdowns, merchant-level deep dives, identifying and fixing misclassifications (e.g. Baloise Life Ltd from Insurance → Investments), grouping merchant name variants (Migros, Digitec/Galaxus), and cutting long-tail noise in visualisations. These patterns directly inform the tool design for Phase 2.

---

### Phase 2: Interactive Chat

With clean, classified data in hand, build the chat application. The architecture is adapted from recipe-vault — an Axum server with an agent loop, SSE streaming, and a vanilla JS frontend.

#### Change 1: `chat-mvp`

The minimum viable chat: a working end-to-end conversation loop where the user can ask about their spending and get answers with inline charts.

**Scope:**
- Axum server serving static files and a chat endpoint
- LLM provider for Claude API (adapted from recipe-vault)
- Agent loop: receive message → call LLM → execute tools → stream response
- SSE streaming of text chunks, tool use indicators, and chart artifacts
- Four read-only analysis tools: `spending_by_category`, `monthly_trend`, `merchant_breakdown`, `income_vs_spending`
- Browser chat UI: message input, streaming text display, Frappe Charts rendering for chart artifacts
- In-memory session management (conversation history)
- No auth (single user, local only)

**Depends on:** Phase 1 complete
**Delivers:** A browser-based chat where you can ask spending questions and get narrative answers with charts.

#### Change 2: `recategorise-tool`

Add the write path. The user can tell the chat to recategorise merchants, and the AI updates the database.

**Scope:**
- `recategorise` tool: updates category on all matching transactions and the merchant cache
- Confirmation flow: the LLM confirms what it's about to change before executing
- Merchant name grouping (e.g. "group all Migros variants together") as a display-level feature

**Depends on:** `chat-mvp`
**Delivers:** Conversational recategorisation — no more running raw SQL to fix categories.

---

### Phase 3: Extended capabilities (future)

These are potential follow-on changes, not yet committed:

- **`sql-escape-hatch`** — Raw SQL tool with read-only guardrails for ad-hoc queries the predefined tools can't handle
- **`csv-import-chat`** — Import new CSV files through the chat interface, with classification progress streaming
- **`review-queue`** — Surface low-confidence transactions for review, either as a dedicated view or as a chat-driven workflow
- **`reusable-agent-lib`** — Extract the agent loop, LLM provider, SSE streaming, and tool framework into a shared Rust crate for use across projects (recipe-vault, budget-analyser, and future tools)
