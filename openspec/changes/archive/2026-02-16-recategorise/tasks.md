## 1. Database Layer

- [x] 1.1 Add `get_transactions_by_category(category: &str) -> Result<Vec<StoredTransaction>>` to `Database` in `src/db.rs` — SELECT all transactions WHERE category = ?, ORDER BY date ASC

## 2. Recategorise Flow

- [x] 2.1 Add `run_recategorise(db, category, categories)` function to `src/review.rs` — groups transactions by normalised merchant key, presents each group with (1) Change category, (2) Skip, (3) Quit menu
- [x] 2.2 On category change: update all transactions in the group, update merchant cache, insert few-shot example (same as review flow)
- [x] 2.3 Print summary on completion or quit (groups reviewed, transactions updated, skipped)

## 3. CLI Integration

- [x] 3.1 Add `recategorise` subcommand to `main.rs` with `--category <name>` flag and optional db_path
- [x] 3.2 Print usage message if `--category` not provided
- [x] 3.3 Print "No transactions found" message if category is empty
- [x] 3.4 Update help text to include the `recategorise` command

## 4. Verification

- [x] 4.1 Verify the project compiles with `cargo build`
- [x] 4.2 Run `cargo test` and fix any failures
