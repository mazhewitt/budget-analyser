## 1. Database Layer

- [x] 1.1 Add `CategoryInfo` struct with `name: String` and `description: String` to `src/db.rs`
- [x] 1.2 Add `categories` table creation (`name TEXT PRIMARY KEY, description TEXT NOT NULL, created_at TEXT NOT NULL`) to `Database::open()`
- [x] 1.3 Add seed logic in `Database::open()` — if `categories` table is empty, insert all 17 defaults from `Category::all()` with their descriptions
- [x] 1.4 Implement `Database::list_categories() -> Result<Vec<CategoryInfo>>` — query all categories ordered by name
- [x] 1.5 Implement `Database::add_category(name, description) -> Result<()>` — insert with duplicate-name error handling

## 2. ClassificationResult Refactor

- [x] 2.1 Change `ClassificationResult.category` from `Category` to `String` in `src/classifier.rs`
- [x] 2.2 Update `Classifier::build_system_prompt()` to accept `&[CategoryInfo]` and generate the category schema from it instead of `Category::schema_for_prompt()`
- [x] 2.3 Update `Classifier::classify()` signature to accept `&[CategoryInfo]` and pass it to `build_system_prompt()`
- [x] 2.4 Update `parse_llm_output()` to return category as `String` directly (no serde enum conversion)
- [x] 2.5 Update `fallback()` to return `"Uncategorised".to_string()` instead of `Category::Uncategorised`

## 3. Database Call Sites

- [x] 3.1 Update `Database::insert_transaction()` — use `classification.category` string directly (remove `.to_string()`)
- [x] 3.2 Update `Database::cache_lookup()` — return category as `String` in `ClassificationResult` (remove serde enum deserialization)
- [x] 3.3 Update `Database::cache_insert()` — use `result.category` string directly (remove `.to_string()`)

## 4. Review Flow

- [x] 4.1 Update `run_review()` in `src/review.rs` to accept `&[CategoryInfo]` parameter
- [x] 4.2 Replace `Category::all()` usage in category selection menu with the passed `CategoryInfo` slice
- [x] 4.3 Update confirm/correct logic to work with `String` category instead of `Category` enum

## 5. Main / CLI Integration

- [x] 5.1 Update `import_file()` and `run_import()` in `src/main.rs` — fetch categories from DB, pass to classifier
- [x] 5.2 Update `run_reclassify()` — fetch categories from DB, pass to classifier
- [x] 5.3 Update review command — fetch categories from DB, pass to `run_review()`
- [x] 5.4 Add `categories list` subcommand — open DB, call `list_categories()`, print results
- [x] 5.5 Add `categories add <name> <description>` subcommand — open DB, call `add_category()`, print confirmation or error
- [x] 5.6 Update usage/help text to include the new `categories` subcommands

## 6. Verification

- [x] 6.1 Verify the project compiles with `cargo build`
- [x] 6.2 Run `cargo test` if tests exist, fix any failures
