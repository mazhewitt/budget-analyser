## ADDED Requirements

### Requirement: spending_by_category tool
The `spending_by_category` tool SHALL query total spending grouped by category, excluding Transfers to avoid double-counting. It accepts optional `year` and `month` filters.

#### Scenario: All-time category breakdown
- **WHEN** `spending_by_category` is called with no parameters
- **THEN** it returns all categories with total spend and transaction count, sorted by spend descending, plus a horizontal bar chart specification

#### Scenario: Filtered by month
- **WHEN** `spending_by_category` is called with `year: 2025, month: 6`
- **THEN** it returns category totals for June 2025 only

#### Scenario: Transfers excluded
- **WHEN** `spending_by_category` is called
- **THEN** the results exclude transactions with category "Transfers"

### Requirement: monthly_trend tool
The `monthly_trend` tool SHALL query monthly spending over time. It accepts an optional `category` filter and optional `year` filter.

#### Scenario: Overall monthly trend
- **WHEN** `monthly_trend` is called with no parameters
- **THEN** it returns total spending per month across all categories (excluding Transfers), plus a vertical bar chart specification

#### Scenario: Category-specific trend
- **WHEN** `monthly_trend` is called with `category: "Groceries"`
- **THEN** it returns monthly spending for Groceries only

#### Scenario: Year filter
- **WHEN** `monthly_trend` is called with `year: 2025`
- **THEN** it returns monthly spending for 2025 only

### Requirement: merchant_breakdown tool
The `merchant_breakdown` tool SHALL query top merchants within a category, ranked by total spend. It accepts a required `category` parameter and optional `top_n` (default 15).

#### Scenario: Top merchants in a category
- **WHEN** `merchant_breakdown` is called with `category: "Groceries"`
- **THEN** it returns the top 15 merchants by spend with total, transaction count, and average per transaction, plus a horizontal bar chart and pie chart specification

#### Scenario: Custom top_n
- **WHEN** `merchant_breakdown` is called with `category: "Dining", top_n: 10`
- **THEN** it returns the top 10 merchants, with remaining spend grouped as "Other"

#### Scenario: Long tail grouped as Other
- **WHEN** there are more merchants than `top_n`
- **THEN** merchants beyond the top N are aggregated into a single "Other" entry in the chart data

### Requirement: income_vs_spending tool
The `income_vs_spending` tool SHALL query monthly income and spending side by side. It accepts an optional `year` filter.

#### Scenario: Full history
- **WHEN** `income_vs_spending` is called with no parameters
- **THEN** it returns monthly income (amount > 0) and spending (amount < 0, excluding Transfers) with net difference, plus a grouped bar chart specification

#### Scenario: Year filter
- **WHEN** `income_vs_spending` is called with `year: 2025`
- **THEN** it returns income vs spending for 2025 only

### Requirement: Tools return text summary and chart specification
Every analysis tool SHALL return a structured result containing both a `summary` string (for the LLM to narrate) and an optional `chart` object (for the frontend to render).

#### Scenario: Tool result structure
- **WHEN** any analysis tool executes successfully
- **THEN** it returns `{"summary": "...", "chart": {...}}` where the summary is a concise text description of the results and the chart is a valid chart specification (or null if no chart applies)

#### Scenario: LLM receives summary only
- **WHEN** a tool result is fed back to the LLM
- **THEN** the LLM receives the `summary` text, NOT the chart specification (chart data goes to the frontend via SSE, not into the LLM context)
