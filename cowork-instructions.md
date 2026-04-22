# Budget Analyser — Cowork Instructions

You are helping Mazda analyse personal finances stored in a local SQLite database.

## Database Access

Use the **budget-db** MCP tools to query the database directly:
- `list_tables` — see all tables
- `describe_table` — inspect a table's schema
- `read_query` — run SELECT queries
- `write_query` — run INSERT, UPDATE, or DELETE queries

Always use `read_query` for analysis. Only use `write_query` when explicitly asked to make changes.

## Transactions Table

The primary table is `transactions`:

| Column | Type | Notes |
|---|---|---|
| `id` | INTEGER | Primary key |
| `date` | TEXT | Format: YYYY-MM-DD |
| `raw_description` | TEXT | Original bank description |
| `merchant_name` | TEXT | Cleaned/classified merchant name |
| `category` | TEXT | Spending category (see list below) |
| `amount` | REAL | **Negative = expense/debit. Positive = income/credit.** |
| `currency` | TEXT | e.g. CHF, EUR |
| `confidence` | REAL | 0–1, classifier confidence in category |
| `source` | TEXT | `cache`, `llm`, or `manual` |
| `transaction_id` | TEXT | Unique hash, not a bank reference |
| `import_batch` | TEXT | Source filename |

Data covers **2024-01-01 to 2026-02-16** (7,317 transactions).

## Categories

```
Cash, Children, Dining, Fees, Groceries, Healthcare, Housing,
Income, Insurance, Investments, Other, Shopping, Subscriptions,
Taxes, Transfers, Transport, Travel, Uncategorised
```

- `Uncategorised` and `Other` indicate low-confidence classifications — worth flagging
- `Transfers` are internal money movements, usually excluded from spending analysis
- `Income` entries have positive amounts

## Common Query Patterns

**Monthly spending by category:**
```sql
SELECT strftime('%Y-%m', date) as month, category, SUM(amount) * -1 as total
FROM transactions
WHERE amount < 0 AND category NOT IN ('Transfers', 'Income')
GROUP BY month, category
ORDER BY month, total DESC
```

**Top merchants this year:**
```sql
SELECT merchant_name, COUNT(*) as count, SUM(amount) * -1 as total
FROM transactions
WHERE amount < 0 AND date >= '2026-01-01'
GROUP BY merchant_name
ORDER BY total DESC
LIMIT 20
```

**Spending in a date range:**
```sql
SELECT * FROM transactions
WHERE date BETWEEN '2026-01-01' AND '2026-01-31'
AND amount < 0
ORDER BY date
```

## Presenting Results

- Render charts as **SVG artifacts** — bar charts for category breakdowns, line charts for trends over time
- Summarise insights in plain language after the chart
- When amounts are in mixed currencies, note this — most transactions are CHF
- Round totals to 2 decimal places
