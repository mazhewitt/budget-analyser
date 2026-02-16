## Why

Users need to redistribute transactions from one category into others. For example, `Transfers` contains both actual account transfers and personal payments (to kids, bill splits with friends) that are really expenditure. The existing `review` command only surfaces low-confidence or Other/Uncategorised transactions — there's no way to review and reassign all transactions within a specific category.

## What Changes

- Add a `get_transactions_by_category()` DB method that returns all transactions for a given category (no confidence filter)
- Add a `recategorise` CLI subcommand that takes a `--category` flag, groups transactions by merchant, and lets the user reassign each group using the existing numbered category menu
- Updates merchant cache and few-shot examples on reassignment (same as review flow)

## Capabilities

### New Capabilities
- `recategorise`: Interactive per-merchant-group reassignment of all transactions in a given category

### Modified Capabilities

## Impact

- **Database** (`src/db.rs`): New query method `get_transactions_by_category()`
- **Review** (`src/review.rs`): New `run_recategorise()` function (reuses the group-and-prompt pattern from `run_review()`)
- **CLI** (`src/main.rs`): New `recategorise --category <name>` subcommand
