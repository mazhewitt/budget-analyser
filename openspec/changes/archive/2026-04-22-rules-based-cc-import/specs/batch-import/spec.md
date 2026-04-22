## MODIFIED Requirements

### Requirement: Per-file import summary
The system SHALL print a summary after each file is imported, showing: filename, total transactions parsed, new insertions, duplicates skipped, cache hits, rules hits, and LLM calls.

#### Scenario: Summary printed after each file
- **WHEN** a file with 24 transactions is imported (20 new, 4 duplicates, 10 cache hits, 8 rules hits, 2 LLM calls)
- **THEN** the system SHALL print a summary line with those counts including a `rules hits` column

#### Scenario: Summary for non-CC file shows zero rules hits
- **WHEN** an account-statement file is imported (no rows route through the rules classifier)
- **THEN** the summary SHALL show `rules hits: 0` while other counters reflect cache and LLM activity

### Requirement: Overall batch summary
After all files are processed, the system SHALL print an overall summary with totals across all files: total files processed, total transactions parsed, total new insertions, total duplicates skipped, total cache hits, total rules hits, and total LLM calls.

#### Scenario: Overall summary after batch import
- **WHEN** 12 CSV files have been imported (mix of credit-card and account-statement formats)
- **THEN** the system SHALL print aggregated totals across all 12 files, including a `rules hits` total

### Requirement: Reusable import function
The per-file import logic SHALL be extracted into a reusable function that accepts a database reference, classifier reference, and CSV file path, and returns import statistics. Both single-file and batch modes SHALL use the same function. The function SHALL detect the CSV format and route credit-card-format rows through the rules classifier with LLM fallback, while routing other formats through the existing cache-then-LLM path.

#### Scenario: Single-file and batch share same logic
- **WHEN** importing a file via single-file mode or batch mode
- **THEN** the same underlying import function SHALL be used, producing identical results for the same input

#### Scenario: Credit card format routes through rules classifier
- **WHEN** the reusable import function processes a file detected as `CsvFormat::CreditCard`
- **THEN** each row SHALL be offered to the rules classifier first; only rows for which rules returns `None` SHALL invoke the cache/LLM path

#### Scenario: Account statement format continues to use cache/LLM
- **WHEN** the reusable import function processes a file detected as `CsvFormat::AccountStatement` or `CsvFormat::Synthetic`
- **THEN** each row SHALL use the existing cache-then-LLM classification path; the rules classifier SHALL NOT be invoked
