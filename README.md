# Budget Analyser

Categorises UBS bank transactions using a local LLM (Ollama) and stores results in SQLite for analysis. Includes an interactive chat interface powered by Claude for natural-language budget queries with inline charts.

Supports three CSV formats:
- UBS account statement export (semicolon-separated, metadata preamble)
- UBS credit card invoice export (semicolon-separated, `sep=;` header)
- Synthetic/test format (comma-separated)

Format is auto-detected from file headers.

## Prerequisites

- Rust toolchain
- [Ollama](https://ollama.com) running locally with a model pulled (default: `qwen3:8b`) — for import/classification
- `ANTHROPIC_API_KEY` environment variable — for the chat server
- Python 3 + `polars` (for the analysis notebook, optional)

```bash
# Pull the default model (for import)
ollama pull qwen3:8b

# Build the project
cargo build --release
```

## Phase 1: Import & Classify (CLI)

### 1. Drop your data files

Place your UBS exports into `data/real_data/`:

```
data/real_data/
  transactions.csv    # account statement export
  invoice.csv         # credit card invoice export
```

This directory is gitignored — your real data will never be committed.

### 2. Import and classify

Make sure Ollama is running, then import your files. You can import a single file or the whole directory:

```bash
# Import everything in the directory at once
cargo run --release -- import data/real_data/

# Or import files individually
cargo run --release -- import data/real_data/transactions.csv
cargo run --release -- import data/real_data/invoice.csv
```

Each transaction is classified by the LLM into one of 16 categories (Groceries, Dining, Transport, Housing, etc.). Results are cached — re-importing the same file skips already-seen transactions.

### 3. Review and correct

Review low-confidence classifications interactively:

```bash
cargo run --release -- review
```

For each flagged transaction you can:
1. Confirm the classification (stores it as a few-shot example for future imports)
2. Change the category
3. Edit the merchant name

Filter the review:

```bash
# Only show a specific category
cargo run --release -- review --category Dining

# Only show transactions since a date
cargo run --release -- review --since 2026-01-01

# Only show transactions below a confidence threshold
cargo run --release -- review --threshold 0.90
```

### 4. Reclassify (optional)

After correcting some transactions, reclassify all low-confidence entries using the improved few-shot examples:

```bash
cargo run --release -- reclassify
```

Same filters as review apply (`--category`, `--since`, `--until`, `--merchant`, `--threshold`).

### 5. Analyse in Jupyter

Open the analysis notebook:

```bash
jupyter notebook analysis.ipynb
```

The notebook connects to `data/budget.db` and provides:
- Spending by category (table + bar chart)
- Monthly spending trend
- Top merchants by total spend
- Monthly spending breakdown by category (stacked bar)
- Classification quality metrics
- Income vs spending comparison

## Phase 2: Interactive Chat Server

An LLM-powered chat interface that lets you ask questions about your spending in natural language, with inline charts rendered automatically.

### Setup

Set your Anthropic API key:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

Optionally configure the bind address and database path (defaults shown):

```bash
export BIND_ADDRESS="127.0.0.1:3000"
export DATABASE_URL="data/budget.db"
```

### Start the server

```bash
cargo run --release -- serve
```

Then open [http://localhost:3000](http://localhost:3000) in your browser.

### What you can ask

The chat agent has access to four analysis tools that query your transaction database directly:

| Tool | Description |
|------|-------------|
| `spending_by_category` | Totals by category, with optional year/month filters |
| `monthly_trend` | Monthly spending over time, with optional category/year filters |
| `merchant_breakdown` | Top merchants within a category |
| `income_vs_spending` | Monthly income vs spending comparison, optional year filter |

Example questions:
- "How much did I spend on groceries last month?"
- "Show me my monthly spending trend for 2025"
- "What are my top merchants in the Dining category?"
- "Compare my income vs spending this year"

The agent will call tools as needed and return results with inline charts (bar, horizontal bar, pie, and grouped bar) rendered via Frappe Charts.

### Architecture

- **Backend**: Rust + Axum, streaming responses via SSE
- **LLM**: Claude (Anthropic API) with an agentic tool-calling loop (up to 10 iterations)
- **Frontend**: Vanilla JavaScript, no build step
- **Charts**: Frappe Charts, delivered as SSE `chart_artifact` events
- **Sessions**: In-memory with 2-hour TTL (conversation history is lost on restart)
