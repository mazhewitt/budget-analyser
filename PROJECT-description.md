# UBS Transaction Categoriser

## Overview

A local-first, AI-first financial tracking application that extracts transaction history from UBS accounts, classifies cryptic merchant strings using local LLMs, and provides a web-based interface for reviewing, correcting, and querying spending data. Designed for a single user running on a Mac Studio with 32GB unified memory.

## Goals

Turn opaque bank statements into a clear, categorised view of spending habits — without sending financial data to any cloud service — through an interactive web interface where the AI does the heavy lifting and the human handles the edge cases.

Specifically:

1. **Import**: Parse UBS CSV exports into a structured format, handling Swiss-specific formatting (apostrophe thousands separators, mixed currency transactions). Detect and skip duplicate imports.
2. **Classify**: Use local LLMs to interpret cryptic merchant descriptions and map them to normalised merchant names and spending categories. Build a merchant cache that learns over time so the LLM handles cold starts while repeated merchants resolve instantly.
3. **Review**: Surface low-confidence and unclassifiable transactions as tasks in the web UI. The user reviews and corrects them; corrections feed back into the system as dynamic few-shot examples, improving future classification accuracy.
4. **Query**: Ask natural language questions about spending ("How much did I spend on travel in October?") through an AI chat interface. The AI uses tool calling to invoke a small set of predefined query functions backed by Polars DataFrames — no raw SQL generation. The AI picks the right tool, the system executes it, and the AI explains the results in plain language.
5. **Store**: Persist everything in SQLite for querying, trend analysis, and long-term history.

## Design Principles

**Local-only processing.** All transaction data stays on the Mac Studio. No cloud APIs, no data leaving the machine. The unified memory architecture is more than sufficient for running quantised models at the scale required.

**Manual CSV import.** Switzerland lacks mandated Open Banking APIs. Rather than building brittle scraping against the UBS web interface, we accept a monthly manual CSV export. This is a reliable, stable data source that won't break when UBS updates their frontend.

**AI-first interaction.** The web interface is designed around AI assistance, not traditional forms and filters. The primary interaction modes are: (1) reviewing AI-suggested classifications, (2) asking questions in natural language, and (3) correcting the AI when it gets things wrong. The UI is lightweight and task-driven — not a dashboard overloaded with charts.

**Two-model strategy.** Use a smaller model (Qwen3 8B) for high-throughput transaction classification where speed matters, and a larger model (Gemma3 27B) for deeper reasoning tasks: understanding natural language queries, selecting the right query tools, and explaining spending patterns. Both run locally via Ollama.

**Tool use over SQL generation.** Rather than having the LLM generate raw SQL (fragile, hard to sandbox, unreliable at 27B scale), the chat interface exposes a small set of coarse-grained query tools via Ollama's tool calling API. The LLM's job is to pick the right tool and parameters — the actual data work happens in Polars DataFrames on the Rust side. This is safer, more reliable, and keeps the tool interface as a clear contract between the AI and the system.

**LLM as cold-start engine, not permanent dependency.** The LLM's job is to bootstrap the merchant cache. Over time, the vast majority of transactions resolve from the cache and the LLM is only invoked for genuinely new merchants. This keeps the system fast and deterministic where it can be.

**Dynamic few-shot learning.** When the user corrects a misclassification, the correction is stored as an example keyed by merchant pattern. These examples are dynamically injected into the LLM prompt, so the system learns from its mistakes without retraining. Over time, the prompt becomes increasingly tailored to this user's specific transaction patterns.

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
| Cash | ATM withdrawals |
| Transfers | Transfers between own accounts, savings |
| Income | Salary, refunds, reimbursements |
| Fees | Bank fees, card fees, foreign exchange fees |
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

- **Rust** for the backend (parsing, orchestration, LLM calls, web server)
- **Axum** for the HTTP server
- **HTMX + server-rendered HTML** for the web UI (Tera or Askama templates). Lightweight, no JS build toolchain. HTMX handles interactivity (review queue updates, chat messages, import progress).
- **SQLite** for storage and persistence (via rusqlite or sqlx)
- **Polars** for in-memory analytics and query execution. Transaction data is loaded from SQLite into DataFrames for fast filtering, grouping, and aggregation.
- **Ollama** for local LLM inference (with tool calling support)
- **Models:** Qwen3 8B for classification, Gemma3 27B for chat/query reasoning

## Query Tools

The chat interface exposes these tools to Gemma3 27B via Ollama's tool calling API. Each tool maps to a Polars query on the Rust side.

| Tool | Parameters | Returns |
|---|---|---|
| `spending_by_category` | `year`, `month?` | Table of categories with totals and transaction counts |
| `spending_by_merchant` | `year`, `month?`, `top_n?` | Table of merchants ranked by spend |
| `spending_trend` | `category?`, `months?` | Monthly time series of spending |
| `search_transactions` | `query?`, `category?`, `min_amount?`, `max_amount?`, `year?`, `month?` | Filtered transaction list |
| `summary` | `year`, `month?` | Overall income, expenses, and balance |

The LLM picks the tool and parameters; the system executes it and returns structured data; the LLM explains the result in plain language. This keeps the interface tight enough for a 27B model to use reliably.

## Web Interface

Three primary views:

**Import** — Upload a UBS CSV, see classification progress in real-time, review summary of what was imported and how transactions were categorised.

**Review Queue** — A task list of transactions the AI couldn't confidently classify. Each task shows the transaction details, the AI's best guess, and lets the user pick the correct category/merchant. Corrections are saved as few-shot examples.

**Chat** — A conversational interface for querying spending data. The user asks natural language questions; Gemma3 27B selects the appropriate query tool via function calling; the system executes it against Polars DataFrames; and the AI explains the results. Responses include both the data and a plain-language explanation.

## Implementation Roadmap

Two phases: first build a reliable data pipeline and get a year of clean data into SQLite, then layer the AI query interface on top.

---

### Phase 1: Data Pipeline

Get the data right first. Feed a year of UBS exports through the system, classify everything with local LLMs, clean up edge cases via CLI, and produce a SQLite database that can be explored directly with Python Polars. No web UI in this phase.

#### Change 1: `sqlite-storage`

Replace the POC's in-memory structures with persistent SQLite storage. Create the `transactions`, `merchant_cache`, and `import_log` tables. Migrate the CSV parser to write into SQLite. Add duplicate detection on import (by transaction_id). Update the category enum to match the expanded 15-category schema. The merchant cache becomes a database table instead of a HashMap — lookups and inserts go through SQLite. Still CLI-driven.

**Depends on:** POC (complete)
**Delivers:** A working import pipeline that persists data and prevents duplicate imports.

#### Change 2: `batch-import`

Support importing multiple CSV files (a year of monthly exports) in a single CLI run. Process them chronologically. The merchant cache builds up across files — early imports train the cache, later imports benefit from it. Print a summary after each file: total transactions, cached hits, LLM classifications, flagged items. At the end, print an overall report across all files.

**Depends on:** `sqlite-storage`
**Delivers:** One command to import a full year of data.

#### Change 3: `cli-review`

Add a CLI review mode for cleaning up flagged transactions. List all transactions where `confidence < threshold` or `category = 'Other'`. For each one, show the description, amount, details, and the LLM's best guess. Accept user input: confirm, re-categorise, or edit merchant name. Update the transaction and merchant cache in SQLite. Support filtering by category, date range, or merchant to review in batches.

**Depends on:** `sqlite-storage`
**Delivers:** Manual correction workflow without needing a web UI.

#### Change 4: `dynamic-few-shot`

Add the `few_shot_examples` table. When a user corrects a transaction in the CLI review, store the correction as a few-shot example keyed by normalised merchant pattern. Update the classification prompt builder to load relevant examples from SQLite and inject them into the system prompt dynamically. Add a `reclassify` CLI command that re-runs the LLM on all flagged transactions using the improved prompt. Re-importing a fresh CSV should now benefit from past corrections.

**Depends on:** `cli-review`
**Delivers:** The system learns from corrections. Classification accuracy improves over time.

#### Phase 1 outcome

After completing Phase 1, you have:
- A SQLite database (`data/budget.db`) containing a full year of classified transactions
- A merchant cache that's been trained on your actual spending patterns
- Few-shot examples from your corrections that improve future classification
- A database you can query directly with Python Polars in a Jupyter notebook or script:

```python
import polars as pl

df = pl.read_database("SELECT * FROM transactions", "sqlite:///data/budget.db")

# Monthly spending by category
df.filter(pl.col("category") == "Transport") \
  .group_by(pl.col("date").dt.month()) \
  .agg(pl.col("amount").sum())
```

---

### Phase 2: AI Query Interface

With clean, classified data in hand, build the web interface and AI query layer.

#### Change 5: `web-skeleton`

Stand up the Axum web server with HTMX and server-rendered templates. Create the basic layout (navigation shell, views for Import, Review, and Chat). Add the CSV upload endpoint — the user uploads a file through the browser, the server runs the existing import + classification pipeline, and returns a summary page. Move the CLI review workflow to a web-based review queue with the same confirm/re-categorise/edit controls.

**Depends on:** Phase 1 complete
**Delivers:** Browser-based import and review workflow.

#### Change 6: `polars-analytics`

Add Polars as a Rust dependency. On server startup (and after each import), load transaction data from SQLite into a Polars DataFrame held in memory. Implement the five query tool functions (`spending_by_category`, `spending_by_merchant`, `spending_trend`, `search_transactions`, `summary`) as Rust functions operating on the DataFrame. Expose these as internal API endpoints (JSON responses). No AI chat yet — this is the data layer that the chat will call.

**Depends on:** `web-skeleton`
**Delivers:** Fast, in-memory analytics engine with a defined tool interface.

#### Change 7: `ai-chat`

Build the chat view. Add an Ollama client configured for Gemma3 27B with tool calling. Define the five query tools as Ollama function definitions. The user types a question; the server sends it to Gemma3 with the tool definitions; if the model calls a tool, the server executes the corresponding Polars function and sends the result back; the model generates a natural language response. Render the conversation in the chat view using HTMX streaming or polling.

**Depends on:** `polars-analytics`
**Delivers:** Natural language querying of spending data through the browser.
