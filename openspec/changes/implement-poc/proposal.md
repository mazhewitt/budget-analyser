## Why

We need to validate two core assumptions before investing in the full budget analyser implementation: (1) whether a local 8B-class LLM can reliably classify cryptic UBS merchant strings, and (2) whether the category schema holds up against real transaction data. The POC will surface accuracy limits, confidence thresholds, and schema gaps early — before we build the CSV pipeline, database, and review UI around them.

## What Changes

- Add a UBS CSV parser that reads the synthetic test dataset (`data/synthetic-ubstransactions-feb2026.csv`)
- Define a transaction category schema with categories covering groceries, transport, utilities, subscriptions, transfers, card payments, and refunds
- Build an LLM classification pipeline that sends transaction descriptions to a local model (via Ollama or similar) and returns merchant name, category, confidence score
- Implement a merchant string normalisation and cache-key strategy to group variants of the same merchant
- Create a ground-truth labelled dataset and an evaluation harness that measures accuracy, flagging rate, and false-confidence rate against the POC success criteria
- Support testing with multiple models (e.g. Llama 3.1 8B, Qwen 2.5 7B)

## Capabilities

### New Capabilities

- `csv-parsing`: Parse UBS monthly export CSV format into structured transaction records
- `category-schema`: Define and manage the transaction category taxonomy (groceries, transport, utilities, etc.)
- `llm-classification`: Send transaction descriptions to a local LLM and parse structured classification responses (merchant name, category, confidence)
- `merchant-cache`: Normalise merchant strings and cache classification results to avoid redundant LLM calls
- `poc-evaluation`: Run classification against labelled ground-truth data and report accuracy, flagging rate, and confidence calibration metrics

### Modified Capabilities

(none — greenfield project, no existing specs)

## Impact

- **Code**: New Rust modules for CSV parsing, LLM HTTP client, merchant normalisation, category schema, and evaluation harness
- **Dependencies**: Will need HTTP client crate (e.g. `reqwest`) for Ollama API calls, CSV crate (`csv`), serialisation (`serde`, `serde_json`), and possibly `sqlx`/`rusqlite` for a lightweight cache store
- **External systems**: Requires a running Ollama instance with at least one 8B-class model pulled locally
- **Data**: Synthetic test CSV already exists at `data/synthetic-ubstransactions-feb2026.csv`; ground-truth labels to be created as part of this change
