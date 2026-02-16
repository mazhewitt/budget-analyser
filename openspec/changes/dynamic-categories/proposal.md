## Why

Spending categories are currently hardcoded as a Rust enum, meaning any new category requires a code change, recompilation, and redeployment. Users should be able to add new spending categories at runtime to adapt the system to their specific financial tracking needs.

## What Changes

- Store category definitions (name + description) in a new `categories` database table, seeded with the current 17 categories on first run
- Add a `list_categories()` function that returns all available categories from the database
- Add an `add_category(name, description)` function that inserts a new category into the database
- Modify the classifier's prompt generation to pull categories from the database instead of the hardcoded enum
- Modify the review flow's category selection menu to use database-sourced categories
- Keep `Uncategorised` as a guaranteed fallback (always present, cannot be removed)
- **BREAKING**: The `Category` enum will no longer be the single source of truth for available categories; code that pattern-matches exhaustively on `Category` variants will need updating

## Capabilities

### New Capabilities
- `dynamic-categories`: Runtime management of spending categories — storing category definitions in the database, listing available categories, and adding new ones without code changes

### Modified Capabilities
- `category-schema`: The category taxonomy will no longer be a fixed Rust enum. Categories are sourced from the database, with the hardcoded set used only for initial seeding. The prompt schema generation must query the database instead of the enum.

## Impact

- **Database**: New `categories` table with migration/seeding logic in `src/db.rs`
- **Categories module** (`src/categories.rs`): Enum retained for seeding defaults but no longer the runtime source of truth; new functions for DB-backed category operations
- **Classifier** (`src/classifier.rs`): Prompt generation reads categories from DB
- **Review** (`src/review.rs`): Category selection menu reads from DB
- **CLI** (`src/main.rs`): May need new subcommands for listing/adding categories
