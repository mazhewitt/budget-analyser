## ADDED Requirements

### Requirement: Database schema with three tables
The system SHALL create a SQLite database with three tables on startup: `transactions`, `merchant_cache`, and `import_log`. Table creation SHALL use `CREATE TABLE IF NOT EXISTS` so the application can start repeatedly without error.

#### Scenario: First startup creates tables
- **WHEN** the application starts and no database file exists
- **THEN** the system SHALL create `data/budget.db` with all three tables

#### Scenario: Subsequent startup preserves data
- **WHEN** the application starts and the database already exists with data
- **THEN** the existing data SHALL be preserved and no tables SHALL be recreated

### Requirement: Transactions table schema
The system SHALL store transactions with columns: `id` (INTEGER PRIMARY KEY), `date` (TEXT, ISO 8601), `raw_description` (TEXT), `amount` (REAL, negative for debits, positive for credits), `currency` (TEXT), `merchant_name` (TEXT), `category` (TEXT), `source` (TEXT: "llm", "cache", or "manual"), `confidence` (REAL), `transaction_id` (TEXT UNIQUE), `import_batch` (TEXT), `created_at` (TEXT, ISO 8601 timestamp).

#### Scenario: Insert a classified transaction
- **WHEN** a transaction is classified and inserted
- **THEN** all fields SHALL be populated and `transaction_id` SHALL be unique

#### Scenario: Amount sign convention
- **WHEN** a debit transaction of CHF 45.00 is stored
- **THEN** the `amount` SHALL be -45.00

#### Scenario: Credit transaction amount
- **WHEN** a credit transaction of CHF 5000.00 is stored
- **THEN** the `amount` SHALL be 5000.00

### Requirement: Merchant cache table schema
The system SHALL store merchant cache entries with columns: `raw_key` (TEXT PRIMARY KEY), `merchant_name` (TEXT), `category` (TEXT), `confidence` (REAL), `source` (TEXT: "llm" or "manual"), `created_at` (TEXT), `updated_at` (TEXT).

#### Scenario: Cache lookup by normalised key
- **WHEN** the system looks up a merchant by normalised key "SBB MOBILE"
- **THEN** it SHALL return the cached merchant name, category, and confidence if the key exists

#### Scenario: Cache insert after LLM classification
- **WHEN** a new merchant is classified by the LLM
- **THEN** the result SHALL be inserted into the merchant_cache table with source "llm"

### Requirement: Import log table schema
The system SHALL store import metadata with columns: `id` (INTEGER PRIMARY KEY), `filename` (TEXT), `row_count` (INTEGER), `imported_at` (TEXT, ISO 8601 timestamp).

#### Scenario: Log a successful import
- **WHEN** a CSV file is successfully imported
- **THEN** an entry SHALL be created in `import_log` with the filename, number of rows imported, and current timestamp

### Requirement: Database connection management
The system SHALL provide a `Database` struct that owns the SQLite connection and exposes methods for all CRUD operations. The database path SHALL default to `data/budget.db` but be configurable.

#### Scenario: Open database with default path
- **WHEN** the application starts without specifying a database path
- **THEN** the database SHALL be opened at `data/budget.db`

#### Scenario: Open database with custom path
- **WHEN** the application specifies a custom database path
- **THEN** the database SHALL be opened at the specified path

### Requirement: Query flagged transactions
The Database SHALL provide a method to query transactions that need review, accepting optional filter parameters (category, date range, merchant substring, confidence threshold). The method SHALL return a list of transaction records matching the flagging criteria AND any applied filters.

#### Scenario: Query with default threshold
- **WHEN** `get_flagged_transactions(threshold=0.80)` is called with no filters
- **THEN** all transactions with confidence < 0.80 OR category in ("Other", "Uncategorised") SHALL be returned

#### Scenario: Query with category filter
- **WHEN** `get_flagged_transactions(threshold=0.80, category="Other")` is called
- **THEN** only transactions with category "Other" that also meet the flagging criteria SHALL be returned

### Requirement: Update transaction classification
The Database SHALL provide a method to update a transaction's merchant_name, category, confidence, and source fields by transaction id. This is used when the user corrects a classification during review.

#### Scenario: Update transaction after correction
- **WHEN** `update_transaction(id, merchant="SBB", category="Transport", confidence=1.0, source="manual")` is called
- **THEN** the transaction record SHALL be updated with the new values

### Requirement: Update merchant cache entry
The Database SHALL provide a method to update or insert a merchant cache entry by normalised key, setting the merchant name, category, confidence, and source. The `updated_at` timestamp SHALL be set to the current time.

#### Scenario: Update existing cache entry after correction
- **WHEN** a cache entry exists for key "SBB MOBILE" and is updated with a new category
- **THEN** the existing entry SHALL be replaced with the new values and `updated_at` SHALL be current
