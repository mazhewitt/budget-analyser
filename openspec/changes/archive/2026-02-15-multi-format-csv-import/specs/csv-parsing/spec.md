## MODIFIED Requirements

### Requirement: Parse UBS monthly export CSV into transaction records

The system SHALL parse CSV files in any of three supported UBS export formats (synthetic, account statement, credit card invoice) and produce a list of typed `Transaction` records. The output struct is identical regardless of source format. Existing synthetic format parsing (11 comma-separated columns, DD.MM.YYYY dates) SHALL continue to work unchanged.

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

## ADDED Requirements

### Requirement: Parse UBS account statement format

The system SHALL parse UBS account statement exports which use semicolon delimiters and contain a metadata preamble. The parser SHALL scan for the header row (starting with `Trade date;`) and skip all lines before it. Date fields SHALL be parsed from `YYYY-MM-DD` format. Description1 SHALL map to `description`. Description2 and Description3 SHALL be concatenated (semicolon-separated, trimming empty parts) and mapped to `details`. The `Transaction no.` column SHALL map to `transaction_id`.

#### Scenario: Parse real account statement with metadata preamble

- **WHEN** the parser is given a file starting with `Account number:;` followed by metadata rows
- **THEN** it SHALL skip all metadata rows and parse only the transaction data rows

#### Scenario: Parse account statement ISO dates

- **WHEN** an account statement row has Trade date "2026-02-12"
- **THEN** the parsed trade_date SHALL be 2026-02-12

#### Scenario: Concatenate description fields

- **WHEN** an account statement row has Description1 "SBB MOBILE; Payment UBS TWINT", Description2 empty, and Description3 "Reason for payment: ...; Transaction no. 999..."
- **THEN** `description` SHALL be "SBB MOBILE; Payment UBS TWINT" and `details` SHALL contain the Description3 content

#### Scenario: Handle quoted fields with embedded semicolons

- **WHEN** a Description1 field is quoted and contains semicolons (e.g. `"Steuerverwaltung;Steinbruchstrasse 18; 7001 Chur; CH"`)
- **THEN** the field SHALL be parsed correctly as a single value

### Requirement: Parse UBS credit card invoice format

The system SHALL parse UBS credit card invoice exports which use semicolon delimiters and start with a `sep=;` line. The parser SHALL skip the `sep=;` line and use the second line as headers. Rows with an empty `Purchase date` (e.g. "Balance brought forward") and rows with booking text "DIRECT DEBIT" SHALL be filtered out. `Booking text` SHALL map to `description`. `Sector` SHALL map to `details`. `Purchase date` SHALL be parsed from DD.MM.YYYY format.

#### Scenario: Filter out non-transaction rows

- **WHEN** a credit card CSV contains a "Balance brought forward" row (empty Purchase date) and a "DIRECT DEBIT" row
- **THEN** those rows SHALL be excluded from the parsed output

#### Scenario: Map credit card fields to Transaction

- **WHEN** a credit card row has Booking text "Coop-1511 Stadelhofen", Sector "Grocery stores", Purchase date "15.01.2026", Debit "16.10", Currency "CHF"
- **THEN** the Transaction SHALL have description "Coop-1511 Stadelhofen", details "Grocery stores", trade_date 2026-01-15, debit Some(16.10), currency "CHF"

#### Scenario: Parse credit card credit/refund rows

- **WHEN** a credit card row has a value in the Credit column and empty Debit column
- **THEN** the Transaction SHALL have credit set and debit as None

### Requirement: Generate stable transaction IDs for credit card rows

The system SHALL generate deterministic transaction IDs for credit card rows by hashing the combination of purchase date, booking text, amount, and booked date. The generated ID SHALL be prefixed with `cc-` to distinguish from native account statement IDs. The same row SHALL always produce the same ID across re-imports.

#### Scenario: Deterministic ID generation

- **WHEN** two imports of the same credit card CSV are run
- **THEN** the same rows SHALL produce identical transaction IDs, and duplicate detection SHALL skip them on re-import

#### Scenario: Distinct IDs for different transactions

- **WHEN** two credit card rows differ in any of purchase date, booking text, amount, or booked date
- **THEN** they SHALL produce different transaction IDs

### Requirement: Realistic test fixture data

The system SHALL include test fixture files that mirror the structure of real UBS exports with anonymised data. There SHALL be one fixture for account statement format and one for credit card invoice format. Fixtures SHALL exercise key parsing edge cases: metadata preamble, filtered rows, quoted fields with semicolons, multiple description fields.

#### Scenario: Account statement test fixture exists

- **WHEN** tests reference the account statement fixture
- **THEN** a file at `data/test/account-statement.csv` SHALL exist with realistic structure (metadata preamble, semicolon delimiter, ISO dates, multi-description fields)

#### Scenario: Credit card test fixture exists

- **WHEN** tests reference the credit card fixture
- **THEN** a file at `data/test/credit-card-invoice.csv` SHALL exist with realistic structure (sep=; header, filtered rows, sector field, DD.MM.YYYY dates)
