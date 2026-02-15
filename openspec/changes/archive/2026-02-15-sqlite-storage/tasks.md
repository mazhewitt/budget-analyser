## Tasks

### 1. Add rusqlite dependency
- [x] Add `rusqlite = { version = "0.31", features = ["bundled"] }` to Cargo.toml

### 2. Update Category enum to 15-category schema
- [x] Replace the 12 POC categories with the 15 production categories: Groceries, Dining, Transport, Housing, Insurance, Healthcare, Shopping, Subscriptions, Children, Travel, Cash, Transfers, Income, Fees, Other (keep Uncategorised as fallback)
- [x] Update `Display`, `description()`, `all()`, and `schema_for_prompt()` to match the new schema
- [x] Update the classifier's system prompt few-shot examples to use new category names where needed (Utilities→Housing/Subscriptions, Taxes→Fees, CardPayments→Transfers)

### 3. Create db.rs module with Database struct
- [x] Create `src/db.rs` with a `Database` struct owning a `rusqlite::Connection`
- [x] Implement `Database::open(path)` that opens/creates the SQLite file and runs `CREATE TABLE IF NOT EXISTS` for all three tables (transactions, merchant_cache, import_log)
- [x] Implement `Database::insert_transaction()` that inserts a classified transaction, using `INSERT OR IGNORE` on `transaction_id` for duplicate detection
- [x] Implement `Database::transaction_exists(transaction_id)` to check for duplicates
- [x] Implement `Database::cache_lookup(raw_key)` to look up a merchant cache entry
- [x] Implement `Database::cache_insert(raw_key, merchant_name, category, confidence, source)` to store a cache entry
- [x] Implement `Database::log_import(filename, row_count)` to record an import run

### 4. Migrate merchant cache from HashMap to SQLite
- [x] Update `cache.rs`: keep `normalise_merchant_key()` and its tests, but remove `MerchantCache` struct (lookups now go through `db.rs`)
- [x] Alternatively, keep `MerchantCache` as a thin wrapper that delegates to `Database` methods — choose whichever is cleaner

### 5. Rewire main.rs orchestration
- [x] Remove `mod evaluator` and delete `src/evaluator.rs`
- [x] Add `mod db` to `main.rs`
- [x] Open database at startup (default `data/budget.db`, configurable via CLI arg)
- [x] For each parsed transaction: check cache via `db.cache_lookup()` → if miss, classify via LLM → insert cache entry via `db.cache_insert()` → insert transaction via `db.insert_transaction()`
- [x] After all transactions: log import via `db.log_import()`
- [x] Print import summary: total parsed, new insertions, duplicates skipped, cache hits vs LLM calls

### 6. Update .gitignore
- [x] Add `/data/*.db` to .gitignore to exclude SQLite database files

### 7. Verify end-to-end
- [x] Run `cargo build` — must compile without errors
- [x] Run against synthetic CSV — transactions should be stored in `data/budget.db`
- [x] Run again with same CSV — all transactions should be skipped as duplicates
- [x] Verify merchant cache persists across runs (second run should show all cache hits)
