## Why

The POC stores everything in memory — transactions are parsed, classified, and evaluated in a single run, then lost. To build a real data pipeline that accumulates a year of classified transactions, we need persistent storage that survives across runs, prevents duplicate imports, and serves as the foundation for all future querying and analysis.

## What Changes

- Add SQLite database with three tables: `transactions`, `merchant_cache`, and `import_log`
- Replace the in-memory `MerchantCache` HashMap with SQLite-backed lookups and inserts
- Migrate the CSV import pipeline to write classified transactions into SQLite
- Add duplicate detection: skip transactions whose `transaction_id` already exists in the database
- Track each import run in `import_log` with filename, row count, and timestamp
- Update the `Category` enum to match the expanded 15-category schema from the project description (add Housing, Healthcare, Children, Travel, Cash, Fees, Other; remove Utilities, Taxes, CardPayments)
- Remove the evaluator module (POC-only, no longer needed in the persistent pipeline)
- Remain CLI-driven — no web UI in this change

## Capabilities

### New Capabilities
- `sqlite-persistence`: Database schema, connection management, and CRUD operations for transactions, merchant cache, and import log tables
- `duplicate-detection`: Prevent re-importing transactions that already exist in the database, tracked via transaction_id and import_log

### Modified Capabilities
- `category-schema`: Expand from 12 POC categories to the 15-category schema defined in PROJECT-description.md
- `merchant-cache`: Change backing store from in-memory HashMap to SQLite table with the same normalisation logic
- `csv-parsing`: After parsing, write transactions to SQLite instead of returning them for in-memory processing

## Impact

- **Dependencies**: Add `rusqlite` (with `bundled` feature) to Cargo.toml
- **Code**: Major refactor of `main.rs` orchestration; new `db.rs` module; modify `cache.rs`, `categories.rs`, `csv_parser.rs`; remove `evaluator.rs`
- **Data**: Creates `data/budget.db` on first run (already gitignored under `/data/*.csv` pattern — need to also gitignore `*.db`)
- **CLI**: Same positional args but output changes from evaluation report to import summary
