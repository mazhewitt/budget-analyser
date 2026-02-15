## Context

The POC validates that local LLMs can classify UBS transactions with >90% accuracy. Everything runs in-memory: CSV is parsed, transactions are classified, results are evaluated, then discarded. To build a year-long data pipeline (Phase 1), we need persistent storage that accumulates data across runs and prevents duplicate imports.

The current codebase has 6 modules: `main.rs`, `csv_parser.rs`, `classifier.rs`, `cache.rs`, `categories.rs`, `evaluator.rs`. The evaluator is POC-only and will be removed. The cache is an in-memory HashMap that will be migrated to SQLite.

## Goals / Non-Goals

**Goals:**
- Persist transactions, merchant cache, and import metadata in SQLite
- Prevent duplicate transaction imports via `transaction_id` uniqueness
- Expand the category enum to the 15-category production schema
- Keep the same CLI-driven workflow (parse → classify → store)
- Maintain the existing merchant cache normalisation logic

**Non-Goals:**
- Web UI (Phase 2)
- Batch multi-file import (next change: `batch-import`)
- CLI review mode (next change: `cli-review`)
- Few-shot examples table (next change: `dynamic-few-shot`)
- Migration from existing data (no production data exists yet)

## Decisions

### SQLite library: `rusqlite` with bundled feature
Using `rusqlite` with `bundled` because it compiles SQLite from source — no system dependency needed. The `bundled` feature ensures consistent SQLite version across environments. We don't need async database access (CLI tool, sequential processing), so rusqlite's synchronous API is the right fit. Alternative: `sqlx` — rejected because async is unnecessary overhead for a CLI pipeline.

### Single `db.rs` module for all database operations
All SQLite interaction goes through one module exposing a `Database` struct with methods for each operation (insert transaction, lookup cache, log import, etc.). This keeps the database as an implementation detail behind a clean API. The `Database` struct owns the `rusqlite::Connection`.

### Schema initialisation via `CREATE TABLE IF NOT EXISTS`
Run schema creation on every startup. No migration framework — the schema is new and there's no existing data to migrate. If schema changes are needed later, we add migrations then.

### Transaction amount stored as single column
UBS CSV has separate debit/credit columns, but for storage we use a single `amount` column: negative for debits, positive for credits. This simplifies queries and matches how spending analysis works. The sign convention is applied during import.

### Category stored as TEXT in SQLite
Store category as its string name (e.g., "Transport") rather than an integer ID. This makes the database human-readable when queried directly with Python Polars, which is the Phase 1 analysis workflow. The `Category` enum handles parsing in Rust.

### Remove evaluator, keep classifier unchanged
The evaluator module (`evaluator.rs`) and ground-truth evaluation flow are POC-only. Remove them. The classifier stays unchanged — it still classifies individual transactions via Ollama. The change is in orchestration: results go to SQLite instead of an evaluation report.

## Risks / Trade-offs

- **[Risk] SQLite file locking with concurrent access** → Not a concern: single-user CLI tool, one process at a time. If Phase 2 web server needs concurrent writes, we handle it then with WAL mode.
- **[Risk] Category enum mismatch between Rust and SQLite** → Mitigated by using string storage and round-tripping through serde. Unknown categories from old data would deserialise as `Other`.
- **[Trade-off] No async database access** → Acceptable for CLI. Revisit if web server needs it in Phase 2.
- **[Trade-off] No migration framework** → Fine for greenfield. Add if schema evolves.
