## Context

This is a greenfield Rust project (`budget-analyser`, edition 2024) with no existing code beyond a placeholder `main.rs`. The POC validates whether a local 8B-class LLM can reliably classify cryptic UBS bank transaction descriptions into spending categories, and whether the category schema holds up against real data.

The synthetic test dataset (`data/synthetic-ubstransactions-feb2026.csv`) contains 24 transactions with 11 columns: trade_date, booking_date, value_date, currency, debit, credit, balance, transaction_id, description, details, footnotes. Merchant strings range from clear ("SBB MOBILE") to cryptic ("Steuerverwaltung EBILL-RECHT" with QRR references, "UBS Switzerland CREDIT CARD STATEMENT").

## Goals / Non-Goals

**Goals:**

- Parse UBS CSV exports into structured transaction records
- Classify transactions via a local Ollama model, returning merchant name, category, and confidence
- Normalise merchant strings and cache results to avoid redundant LLM calls
- Evaluate classification accuracy against manually-labelled ground truth
- Support swapping models (Llama 3.1 8B, Qwen 2.5 7B) for comparison

**Non-Goals:**

- Interactive review UI (post-POC)
- Full SQLite database schema and import pipeline (post-POC)
- Batch processing orchestration (post-POC)
- Production error handling or retry logic
- Support for non-UBS CSV formats

## Decisions

### D1: Use `reqwest` for Ollama API calls (not `ollama-rs` crate)

The Ollama REST API surface needed is minimal — a single `POST /api/chat` endpoint with JSON mode. Using `reqwest` + `serde_json` directly gives full control over prompt formatting and response parsing without adding a wrapper crate's transitive dependencies.

**Alternative considered**: `ollama-rs` — actively maintained but adds abstraction we don't need for one endpoint.

### D2: In-memory `HashMap` cache (not SQLite)

For the POC, a `HashMap<String, CacheEntry>` keyed on normalised merchant strings is sufficient. The dataset is 24 transactions. SQLite adds complexity without POC benefit.

**Alternative considered**: `rusqlite` — appropriate for the full implementation but overkill here.

### D3: Merchant normalisation strategy

Normalise by: uppercase, strip trailing digit sequences (reference numbers, card fragments), collapse whitespace, remove known suffixes ("EBILL-RECHT", "EBILL", "KARTE XXXX"). This groups variants like "SBB MOBILE" across different transaction IDs while keeping distinct merchants separate (e.g., "COOP PRONTO" vs "COOP CITY").

**Alternative considered**: Regex-only approach — too brittle for the variety of UBS formats. A sequence of normalisation steps is more maintainable.

### D4: Structured JSON output from LLM

Request JSON mode from Ollama (`"format": "json"` in the API request) and define the expected schema in the system prompt. The LLM returns `{ "merchant": "...", "category": "...", "confidence": 0.0-1.0 }`. Parse with `serde_json` and validate against the category enum.

**Alternative considered**: Free-text output with regex parsing — fragile and harder to validate.

### D5: Ground-truth labels as a TOML file

Store manual labels in `data/ground-truth.toml` mapping transaction_id → expected category and merchant. TOML is human-readable and easy to edit. Loaded at evaluation time and compared against LLM output.

**Alternative considered**: Inline labels in CSV — pollutes the test data file.

### D6: Module structure

```
src/
  main.rs          — CLI entry point, orchestrates POC run
  csv_parser.rs    — UBS CSV parsing
  categories.rs    — Category enum and schema definition
  classifier.rs    — Ollama API client, prompt construction, response parsing
  cache.rs         — Merchant normalisation and in-memory cache
  evaluator.rs     — Accuracy, flagging rate, confidence metrics
```

Single binary, no library crate. Each module has a clear responsibility.

## Risks / Trade-offs

- **[LLM accuracy below 70%]** → Fall back to hybrid approach: LLM extracts/normalises merchant name only, category assignment is rule-based. The modular design supports this pivot.
- **[Ollama not running or model not pulled]** → POC will fail at runtime with a connection error. Documented as a prerequisite, not handled programmatically.
- **[JSON mode not supported by chosen model]** → Some older models don't respect `"format": "json"`. Mitigation: test with models known to support it (Llama 3.1+, Qwen 2.5+). Fall back to text parsing if needed.
- **[Normalisation false collisions]** → Aggressive stripping could merge distinct merchants. Mitigation: the evaluation harness will surface these as misclassifications, allowing iterative tuning.
- **[24 transactions may be too few]** → Small sample size limits statistical confidence. Acceptable for POC; the full implementation will use real monthly exports.
