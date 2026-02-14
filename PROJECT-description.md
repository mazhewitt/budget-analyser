# UBS Transaction Categoriser

## Overview

A local-first financial tracking pipeline that extracts transaction history from UBS accounts, classifies cryptic merchant strings using a local LLM, and builds a queryable spending database over time. Designed for a single user running on a Mac Studio with 32GB unified memory.

## Goals

The primary goal is to turn opaque bank statements into a clear, categorised view of spending habits without sending financial data to any cloud service.

Specifically:

1. Parse UBS CSV exports into a structured format, handling Swiss-specific formatting (e.g. apostrophe thousands separators, mixed currency transactions).
2. Use a local LLM (8B–14B parameter range) to interpret cryptic merchant description strings and map them to a two-level schema: normalised merchant name and spending category.
3. Build a merchant resolution cache that learns over time — the LLM handles cold starts, but repeated merchants are resolved instantly from the cache without an LLM call.
4. Flag low-confidence classifications for manual review, and feed those manual corrections back into the cache.
5. Store everything in SQLite for querying and trend analysis.

## Design Principles

**Local-only processing.** All transaction data stays on the Mac Studio. No cloud APIs, no data leaving the machine. The unified memory architecture is more than sufficient for running quantised models at the scale required (a few hundred transactions per month).

**Manual CSV import.** Switzerland lacks mandated Open Banking APIs. Rather than building brittle scraping against the UBS web interface, we accept a monthly manual CSV export. This is a reliable, stable data source that won't break when UBS updates their frontend.

**LLM as cold-start engine, not permanent dependency.** The LLM's job is to bootstrap the merchant cache. Over time, the vast majority of transactions resolve from the cache and the LLM is only invoked for genuinely new merchants. This keeps the system fast and deterministic where it can be.

**Amount-aware classification.** Monetary values are essential context for accurate categorisation. A 6 CHF transaction at Coop Pronto is a coffee; a 145 CHF transaction at the same merchant is a grocery shop. The LLM receives the full transaction including amount.

## Category Schema

A flat, single-level category list. Kept deliberately simple — over-engineering the taxonomy is a common trap that makes classification harder without adding analytical value.

| Category | Covers |
|---|---|
| Groceries | Supermarkets, food shops, bakeries, butchers |
| Dining | Restaurants, cafés, bars, takeaway, fast food |
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

Three core tables:

**transactions** — every imported row from UBS, with its categorisation.

- `id`, `date`, `raw_description`, `amount`, `currency`, `merchant_name`, `category`, `source` (llm/manual/cache), `confidence`, `import_batch`, `created_at`

**merchant_cache** — the learned mapping from raw merchant strings to normalised names and categories.

- `raw_key` (normalised lookup key), `merchant_name`, `category`, `confidence`, `source` (llm/manual), `created_at`, `updated_at`

**import_log** — tracks each CSV import to prevent duplicates.

- `id`, `filename`, `row_count`, `imported_at`

## Tech Stack

- **Rust** for the pipeline (parsing, orchestration, LLM calls)
- **SQLite** for storage
- **Ollama or MLX** for local LLM inference, exposing an OpenAI-compatible API
- **Model:** Gemma3 in Ollama
