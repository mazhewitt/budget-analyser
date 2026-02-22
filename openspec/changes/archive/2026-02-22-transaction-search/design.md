## Context

The existing tool infrastructure in `src/tools/mod.rs` follows a clear pattern: each tool is a struct for input deserialization, an async function that queries SQLite and returns `ToolOutput` (summary text + optional charts), and a registration entry in `ToolRegistry::new()`. Both new tools follow this exact pattern.

The `transactions` table has all the fields we need: `date`, `amount`, `merchant_name`, `raw_description`, `category`. The search needs to hit both `merchant_name` and `raw_description` because bank data is messy — the merchant classifier normalises to `merchant_name`, but the raw string often has the actual product/service name (e.g. `GOOGLE *Pokemon GO`).

## Goals / Non-Goals

**Goals:**
- Enable searching transactions by partial merchant name or description
- Return both summary-level and transaction-level views
- Follow the same tool pattern as existing tools (consistency)
- Both tools share the same search interface (search term + category/year/month filters)

**Non-Goals:**
- Full-text search or ranking (LIKE is sufficient for this data size)
- Aggregation or analytics beyond basic summary stats (that's what the existing tools do)
- Pagination with cursors/offsets (simple limit is enough)

## Decisions

### 1. Case-insensitive LIKE on both merchant_name and raw_description

Search with `WHERE (merchant_name LIKE '%term%' OR raw_description LIKE '%term%')` using SQLite's case-insensitive LIKE (via `COLLATE NOCASE` or wrapping in `LOWER()`).

**Why both fields?** The `merchant_name` is normalised and sometimes loses specificity (e.g. just "Google"). The `raw_description` preserves the bank's original string which often contains the product name. Searching both catches everything.

**Why not full-text search?** With ~7,300 transactions, LIKE is fast enough. FTS would add schema changes and complexity for no user-visible benefit.

### 2. search_transactions returns summary + trend chart

Returns: total spend, transaction count, average transaction, date range, merchant name variants found, plus a monthly bar chart of spending over time for the matched transactions.

The summary gives the LLM enough to narrate ("You spent CHF 745 on Pokemon Go across 24 transactions...") and the chart shows the spending pattern visually.

### 3. list_transactions returns individual rows, default limit 50

Returns raw transaction rows: date, amount, merchant_name, raw_description. Ordered by date descending (most recent first). Default limit of 50 to prevent flooding the LLM context for high-volume merchants.

**Why 50?** Enough to show a meaningful list, small enough to stay within LLM context. The summary from `search_transactions` always shows the true total regardless of limit.

### 4. Same filter interface for both tools

Both accept: `search` (required), `category` (optional), `year` (optional), `month` (optional). Plus `list_transactions` has `limit` (optional, default 50).

This means the LLM can reuse the same search term when moving from summary to detail view, which is the natural conversation flow now that we have chat context.

## Risks / Trade-offs

- **Large result sets in list_transactions** → Mitigated by default limit of 50. The LLM can pass a smaller limit if the user only wants recent transactions.
- **Merchant name variants appearing as separate entries** → `search_transactions` explicitly lists the distinct merchant names found, so the LLM can explain the grouping to the user.
