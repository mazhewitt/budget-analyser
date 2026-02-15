## MODIFIED Requirements

### Requirement: Parse UBS monthly export CSV into transaction records

The system SHALL parse CSV files in UBS monthly export format (11 columns: trade_date, booking_date, value_date, currency, debit, credit, balance, transaction_id, description, details, footnotes) and produce a list of typed transaction records. Date fields SHALL be parsed from DD.MM.YYYY format. Debit and credit fields SHALL be parsed as optional f64 values (one is always empty). The description field SHALL be preserved verbatim for downstream classification. After parsing, classified transactions SHALL be written to the SQLite database via the Database module rather than held in memory for evaluation.

#### Scenario: Parse synthetic test dataset

- **WHEN** the parser is given `data/synthetic-ubstransactions-feb2026.csv`
- **THEN** it SHALL return 24 transaction records with all fields correctly populated

#### Scenario: Handle empty optional fields

- **WHEN** a transaction row has an empty credit field (debit transaction) or empty debit field (credit transaction)
- **THEN** the corresponding field SHALL be `None` and the other SHALL contain the parsed amount

#### Scenario: Parse date fields in DD.MM.YYYY format

- **WHEN** a transaction has trade_date "12.02.2026"
- **THEN** the parsed date SHALL be 2026-02-12

#### Scenario: Preserve description verbatim

- **WHEN** a transaction has description "Steuerverwaltung EBILL-RECHT"
- **THEN** the transaction record's description field SHALL be exactly "Steuerverwaltung EBILL-RECHT" (no trimming or normalisation)
