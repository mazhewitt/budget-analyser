## Why

The chat tools currently stop at category-level and top-merchant-level summaries. Users can't drill into a specific merchant ("How much did I spend on Pokemon Go?") or see individual transactions. The existing `merchant_breakdown` tool groups by exact `merchant_name`, so variant names (e.g. "Google" vs "Google Pokemon GO") appear as separate entries, and there's no way to search across `raw_description` where the real detail lives. Users need to search and explore at the transaction level.

## What Changes

- Add `search_transactions` tool: fuzzy text search across `merchant_name` and `raw_description`, returning a summary (total spend, count, average, date range, merchant name variants) plus a monthly spending trend chart. Accepts optional `category`, `year`, and `month` filters.
- Add `list_transactions` tool: same search interface, returning the individual transaction rows (date, amount, merchant, raw description). Accepts an optional `limit` parameter for pagination.
- Both tools use case-insensitive partial matching (SQL LIKE) on a single search term.

## Capabilities

### New Capabilities
- `transaction-search`: Search and list individual transactions by merchant name or description, with optional category/date filters

### Modified Capabilities

## Impact

- **Backend**: Two new tool functions in `src/tools/mod.rs`, two new entries in `ToolRegistry`
- **Frontend**: No changes — tools return text summaries and chart specs via existing SSE protocol
- **LLM context**: `list_transactions` returns individual rows to the LLM. For merchants with many transactions (e.g. Migros with 300+), the `limit` parameter keeps the response bounded. Default limit of 50.
