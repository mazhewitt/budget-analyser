## Context

Categories are defined as a hardcoded Rust enum (`Category`) in `src/categories.rs` with 17 variants. The enum provides `Display`, `description()`, `all()`, and `schema_for_prompt()`. Categories are stored as strings in the database (transactions, merchant_cache, few_shot_examples tables) but parsed back into the enum via serde. The classifier builds LLM prompts using `Category::schema_for_prompt()`, and the review flow lists categories via `Category::all()`.

Since categories are already stored as strings in the DB, the data layer is already flexible — the rigidity is in the Rust enum being the sole source of truth at runtime.

## Goals / Non-Goals

**Goals:**
- Store category definitions (name + description) in a `categories` table
- Seed the table with the existing 17 categories on first run
- Provide `list_categories()` and `add_category()` functions on `Database`
- Replace `Category::schema_for_prompt()` and `Category::all()` usages with DB-sourced equivalents
- Add CLI subcommands: `categories list` and `categories add <name> <description>`
- Ensure `Uncategorised` always exists as a fallback

**Non-Goals:**
- Removing categories (explicitly deferred)
- Editing existing category descriptions
- Migrating away from the `Category` enum entirely — it stays for seeding and as a fallback for deserialization
- UI/TUI for category management beyond CLI

## Decisions

### 1. New `categories` table with auto-seeding

Add a `categories` table with columns: `name TEXT PRIMARY KEY, description TEXT NOT NULL, created_at TEXT NOT NULL`. On `Database::open()`, after creating the table, check if it's empty and seed from `Category::all()` with their descriptions.

**Why**: Using `CREATE TABLE IF NOT EXISTS` + conditional seeding fits the existing pattern in `db.rs`. The name is the primary key since category names must be unique and are used as string references everywhere.

**Alternative considered**: A migration system — overkill for a single table addition.

### 2. New `CategoryInfo` struct replaces enum in dynamic contexts

Introduce a simple struct `CategoryInfo { name: String, description: String }` for runtime use. The `Category` enum remains for seeding defaults and for serde deserialization fallback in `cache_lookup` and `parse_llm_output`.

**Why**: A struct with owned strings is the natural representation for DB-sourced data. The enum stays because removing it would be a much larger refactor touching serde, and it still has value as the "known defaults".

### 3. `ClassificationResult.category` changes from `Category` to `String`

The `category` field on `ClassificationResult` becomes a `String` instead of `Category`. This is the key breaking change. Call sites that match on `category` (e.g., `result.category.to_string()`) simplify to just using the string directly.

**Why**: With dynamic categories, the LLM can return category names that don't exist in the enum. Keeping the enum as the type would require either a catch-all variant or constant enum updates — defeating the purpose. String is already how categories are stored in the DB.

**Alternative considered**: An `enum Category { Known(KnownCategory), Custom(String) }` wrapper — adds complexity with no real benefit since we already store strings.

### 4. Classifier prompt generation takes a category list parameter

`Classifier::classify()` and `build_system_prompt()` accept a `&[CategoryInfo]` parameter instead of calling `Category::schema_for_prompt()` internally. The caller (main.rs) fetches categories from the DB once and passes them through.

**Why**: Keeps the classifier stateless and testable. The DB query happens once per import/reclassify run, not per transaction.

### 5. Review flow reads categories from DB

`run_review()` receives `&[CategoryInfo]` (fetched by the caller) and uses it for the category selection menu instead of `Category::all()`.

**Why**: Same pattern as classifier — keep DB access in the caller, pass data down.

### 6. CLI subcommands for category management

Add `categories list` and `categories add <name> <description>` subcommands to main.rs. These are simple DB operations with console output.

**Why**: Minimal surface area. The user asked for list and add, no remove.

## Risks / Trade-offs

- **LLM may not recognise custom categories well** → Mitigation: descriptions in the prompt give the LLM context. Users can also correct via review flow, which feeds few-shot examples.
- **Breaking change to `ClassificationResult`** → Mitigation: contained within the codebase, no external API. All call sites updated in this change.
- **Category name collisions** → Mitigation: `name` is `PRIMARY KEY` — the DB rejects duplicates. The `add_category` function returns a clear error.
- **Uncategorised could be accidentally re-added** → Mitigation: seeding is idempotent (only runs when table is empty), and `Uncategorised` is part of the seed set.
