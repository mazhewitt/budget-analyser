## ADDED Requirements

### Requirement: search_transactions tool returns spending summary for matched transactions
The `search_transactions` tool SHALL accept a `search` parameter (required) and optional `category`, `year`, and `month` filters. It SHALL perform a case-insensitive partial match on both `merchant_name` and `raw_description`. It SHALL return a text summary containing: total spend, transaction count, average transaction amount, date range, and distinct merchant name variants found. It SHALL return a monthly bar chart of spending over time for matched transactions.

#### Scenario: Search by merchant name
- **WHEN** the user calls `search_transactions` with `search: "pokemon"`
- **THEN** the tool returns transactions where `merchant_name` OR `raw_description` contains "pokemon" (case-insensitive), with a summary showing total spend, count, average, date range, and merchant name variants

#### Scenario: Search with category filter
- **WHEN** the user calls `search_transactions` with `search: "google"` and `category: "Subscriptions"`
- **THEN** only transactions matching both the search term AND the category are included in the summary

#### Scenario: Search with year filter
- **WHEN** the user calls `search_transactions` with `search: "migros"` and `year: 2025`
- **THEN** only transactions from 2025 matching the search term are included

#### Scenario: No results found
- **WHEN** the user calls `search_transactions` with a search term that matches no transactions
- **THEN** the tool returns a summary stating no transactions were found, with no chart

### Requirement: list_transactions tool returns individual transaction rows
The `list_transactions` tool SHALL accept a `search` parameter (required) and optional `category`, `year`, `month`, and `limit` filters. It SHALL perform the same case-insensitive partial match as `search_transactions`. It SHALL return individual transaction rows containing: date, amount, merchant_name, and raw_description. Results SHALL be ordered by date descending. The default limit SHALL be 50.

#### Scenario: List transactions for a merchant
- **WHEN** the user calls `list_transactions` with `search: "pokemon"`
- **THEN** the tool returns up to 50 individual transaction rows ordered by date descending, each showing date, amount, merchant_name, and raw_description

#### Scenario: List with custom limit
- **WHEN** the user calls `list_transactions` with `search: "coop"` and `limit: 10`
- **THEN** only the 10 most recent matching transactions are returned

#### Scenario: List indicates when results are truncated
- **WHEN** the matched transactions exceed the limit
- **THEN** the summary text indicates how many total transactions exist vs how many are shown (e.g. "Showing 50 of 300 transactions")

### Requirement: Both tools use the same search interface
Both `search_transactions` and `list_transactions` SHALL accept the same set of search and filter parameters: `search` (required string), `category` (optional string), `year` (optional integer), `month` (optional integer). The search matching logic SHALL be identical between both tools.

#### Scenario: Consistent results between search and list
- **WHEN** the user calls `search_transactions` with `search: "pokemon"` and then `list_transactions` with the same parameters
- **THEN** the transaction count in the search summary matches the total number of transactions that would be returned by list (before limit is applied)
