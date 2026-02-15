## ADDED Requirements

### Requirement: Skip duplicate transactions by transaction_id
The system SHALL check whether a transaction's `transaction_id` already exists in the `transactions` table before inserting. If it exists, the transaction SHALL be skipped silently. The import summary SHALL report how many transactions were skipped as duplicates.

#### Scenario: First import of a CSV file
- **WHEN** a CSV file is imported and no transactions from it exist in the database
- **THEN** all transactions SHALL be inserted

#### Scenario: Re-import of the same CSV file
- **WHEN** a CSV file is imported and all its transaction_ids already exist in the database
- **THEN** zero new transactions SHALL be inserted and the summary SHALL show all were duplicates

#### Scenario: Partial overlap between imports
- **WHEN** a CSV file contains some transactions already in the database and some new ones
- **THEN** only the new transactions SHALL be inserted and duplicates SHALL be skipped

### Requirement: Import log tracks each import run
The system SHALL record each import attempt in the `import_log` table, including the filename and the count of newly inserted rows (not duplicates). This provides an audit trail of what was imported and when.

#### Scenario: Import log entry after successful import
- **WHEN** 24 transactions are parsed and 20 are new (4 duplicates)
- **THEN** the import_log entry SHALL record `row_count = 20`
