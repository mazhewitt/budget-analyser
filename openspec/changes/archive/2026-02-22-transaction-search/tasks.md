## 1. Shared search infrastructure

- [x] 1.1 Add `TransactionSearchInput` struct with `search`, `category`, `year`, `month` fields in `src/tools/mod.rs`
- [x] 1.2 Add a helper function `build_search_query` that constructs the WHERE clause with case-insensitive LIKE on `merchant_name` and `raw_description`, plus optional category/year/month filters

## 2. search_transactions tool

- [x] 2.1 Add `search_transactions` tool definition to `ToolRegistry::new()` with input schema
- [x] 2.2 Implement `search_transactions` function: query for total spend, count, avg, date range, and distinct merchant names
- [x] 2.3 Add monthly trend chart to `search_transactions` output (bar chart of spending by month for matched transactions)
- [x] 2.4 Register `search_transactions` in the `run()` match arm

## 3. list_transactions tool

- [x] 3.1 Add `ListTransactionsInput` struct extending search fields with `limit` (default 50)
- [x] 3.2 Add `list_transactions` tool definition to `ToolRegistry::new()` with input schema
- [x] 3.3 Implement `list_transactions` function: query individual rows (date, amount, merchant_name, raw_description) ordered by date DESC with limit
- [x] 3.4 Include truncation notice in summary when total results exceed limit
- [x] 3.5 Register `list_transactions` in the `run()` match arm

## 4. Verification

- [x] 4.1 Build and start the server, test `search_transactions` via curl with search term "pokemon"
- [x] 4.2 Test `list_transactions` via curl with search term "pokemon" and verify individual rows returned
- [x] 4.3 Test with category and year filters to verify filtering works
