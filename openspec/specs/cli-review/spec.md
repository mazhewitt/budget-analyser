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
For each flagged transaction, the system SHALL present options: (1) Confirm, (2) Change category, (3) Edit merchant, (4) Skip, (5) Quit. When the user confirms or corrects a transaction, the system SHALL store the correction as a few-shot example in addition to updating the transaction and merchant cache.

#### Scenario: Correction stored as few-shot example
- **WHEN** the user corrects a transaction's category from "Other" to "Fees"
- **THEN** a few-shot example SHALL be stored with the normalised merchant pattern, raw description, corrected merchant name, and corrected category

#### Scenario: Confirm stores few-shot example
- **WHEN** the user confirms an LLM classification
- **THEN** a few-shot example SHALL be stored with the confirmed merchant and category

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
