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
