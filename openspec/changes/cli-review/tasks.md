## Tasks

### 1. Add Database query and update methods
- [x] Add `Database::get_flagged_transactions()` that queries transactions where confidence < threshold OR category in ("Other", "Uncategorised"), with optional filters for category, since/until dates, and merchant substring
- [x] Add `Database::update_transaction()` that updates merchant_name, category, confidence, and source for a given transaction id
- [x] The existing `cache_insert()` method already handles upsert — no changes needed for cache updates

### 2. Create review.rs module
- [x] Create `src/review.rs` with a `ReviewFilters` struct (category, since, until, merchant, threshold)
- [x] Implement the main review loop: fetch flagged transactions, iterate with interactive prompt
- [x] For each transaction, display: raw_description, merchant_name, category, confidence, amount, date
- [x] Present menu: (1) Confirm, (2) Change category, (3) Edit merchant, (4) Skip, (5) Quit
- [x] Read user input from stdin for menu selection
- [x] For "Change category": display numbered category list, accept selection
- [x] For "Edit merchant": read new merchant name from stdin
- [x] On confirm/change/edit: update transaction via `db.update_transaction()` and cache via `db.cache_insert()`
- [x] Track review stats: total reviewed, confirmed, corrected, skipped
- [x] Print review summary at the end

### 3. Add subcommand routing to main.rs
- [x] Refactor `main.rs` to check the first arg: "import", "review", or a file/directory path
- [x] "import" subcommand: parse remaining args (path, db_path, model, endpoint) and run existing import logic
- [x] "review" subcommand: parse remaining args (--category, --since, --until, --merchant, --threshold, db_path) and call `review::run_review()`
- [x] Backward compatibility: if first arg is a file/directory path (not "import" or "review"), treat as import
- [x] Add `mod review` to main.rs

### 4. Verify end-to-end
- [x] Run `cargo build` — must compile without errors
- [x] Run `cargo run -- import data/synthetic-ubstransactions-feb2026.csv` — should work as before
- [x] Run `cargo run -- review` — should show flagged transactions from the database
- [x] Verify corrections persist: after correcting a transaction, re-running review should not show it again (if confidence was raised to 1.0)
