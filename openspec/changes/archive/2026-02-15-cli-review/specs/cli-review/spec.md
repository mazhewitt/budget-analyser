## ADDED Requirements

### Requirement: List flagged transactions for review
The system SHALL query the database for transactions that need review. A transaction is flagged if its confidence is below a configurable threshold (default 0.80), OR its category is "Other", OR its category is "Uncategorised". Flagged transactions SHALL be presented one at a time in an interactive CLI loop.

#### Scenario: Transactions below confidence threshold
- **WHEN** the review command is run with threshold 0.80
- **THEN** all transactions with confidence < 0.80 SHALL be listed for review

#### Scenario: Transactions categorised as Other or Uncategorised
- **WHEN** the review command is run
- **THEN** all transactions with category "Other" or "Uncategorised" SHALL be listed regardless of confidence

#### Scenario: No flagged transactions
- **WHEN** no transactions match the flagging criteria
- **THEN** the system SHALL print "No transactions to review" and exit cleanly

### Requirement: Display transaction details for review
For each flagged transaction, the system SHALL display: the raw description, merchant name, category, confidence score, amount, date, and details field. The display SHALL be clear enough for the user to make a correction decision.

#### Scenario: Show transaction context
- **WHEN** a flagged transaction is presented for review
- **THEN** the system SHALL show raw_description, merchant_name, category, confidence, amount, and date

### Requirement: Accept user corrections
For each flagged transaction, the system SHALL present options: (1) Confirm — accept the current classification, (2) Change category — select from the category list, (3) Edit merchant — type a new merchant name, (4) Skip — move to next without changes, (5) Quit — exit review. User input SHALL be read from stdin.

#### Scenario: Confirm classification
- **WHEN** the user selects "Confirm"
- **THEN** the transaction's confidence SHALL be updated to 1.0 and source to "manual"

#### Scenario: Change category
- **WHEN** the user selects "Change category" and picks "Transport"
- **THEN** the transaction's category SHALL be updated to "Transport", confidence to 1.0, and source to "manual"

#### Scenario: Edit merchant name
- **WHEN** the user selects "Edit merchant" and types "Coop Pronto"
- **THEN** the transaction's merchant_name SHALL be updated to "Coop Pronto", confidence to 1.0, and source to "manual"

#### Scenario: Skip transaction
- **WHEN** the user selects "Skip"
- **THEN** the transaction SHALL not be modified and the next flagged transaction SHALL be shown

#### Scenario: Quit review
- **WHEN** the user selects "Quit"
- **THEN** the review loop SHALL exit immediately, preserving all changes made so far

### Requirement: Persist corrections to transaction and merchant cache
When a user confirms or corrects a transaction, the system SHALL update the transaction record in the `transactions` table AND update the corresponding entry in the `merchant_cache` table using the normalised merchant key. This ensures future imports of the same merchant benefit from the correction.

#### Scenario: Correction updates merchant cache
- **WHEN** the user corrects a transaction's category from "Other" to "Transport"
- **THEN** the merchant_cache entry for that transaction's normalised key SHALL be updated with the new category, confidence 1.0, and source "manual"

### Requirement: Filter review queue
The review command SHALL support optional filters that narrow the set of flagged transactions: `--category <name>` (exact match), `--since <YYYY-MM-DD>` (date >= value), `--until <YYYY-MM-DD>` (date <= value), `--merchant <substring>` (case-insensitive substring match on merchant_name). Filters SHALL be combined with AND logic.

#### Scenario: Filter by category
- **WHEN** the user runs `review --category Other`
- **THEN** only flagged transactions with category "Other" SHALL be shown

#### Scenario: Filter by date range
- **WHEN** the user runs `review --since 2026-01-01 --until 2026-03-31`
- **THEN** only flagged transactions within that date range SHALL be shown

#### Scenario: No filters applied
- **WHEN** the user runs `review` without filters
- **THEN** all flagged transactions SHALL be shown

### Requirement: Review summary
After the review loop completes (user finishes all or quits early), the system SHALL print a summary showing: total reviewed, confirmed, corrected (category or merchant changed), and skipped.

#### Scenario: Summary after completing review
- **WHEN** the user reviews 10 transactions (3 confirmed, 5 corrected, 2 skipped)
- **THEN** the system SHALL print "Reviewed: 10, Confirmed: 3, Corrected: 5, Skipped: 2"

### Requirement: Subcommand routing
The CLI SHALL support subcommands: `import` (existing import functionality) and `review` (new review functionality). If the first argument is neither subcommand keyword and is a valid file/directory path, it SHALL be treated as `import` for backward compatibility.

#### Scenario: Explicit import subcommand
- **WHEN** the user runs `cargo run -- import data/exports/`
- **THEN** the system SHALL run the import pipeline

#### Scenario: Explicit review subcommand
- **WHEN** the user runs `cargo run -- review`
- **THEN** the system SHALL run the review workflow

#### Scenario: Backward compatible path argument
- **WHEN** the user runs `cargo run -- data/file.csv`
- **THEN** the system SHALL treat it as an import command
