## ADDED Requirements

### Requirement: Accept directory or single file as input
The system SHALL detect whether the first CLI argument is a file or a directory. If it is a file, the system SHALL import that single file. If it is a directory, the system SHALL discover all `*.csv` files in that directory and import them in sequence.

#### Scenario: Single file input (backward compatible)
- **WHEN** the user runs `cargo run -- data/export.csv`
- **THEN** the system SHALL import that single CSV file, identical to the current behavior

#### Scenario: Directory input with multiple CSV files
- **WHEN** the user runs `cargo run -- data/exports/`
- **THEN** the system SHALL discover all `*.csv` files in that directory and import each one

#### Scenario: Directory with no CSV files
- **WHEN** the user runs `cargo run -- data/empty-dir/`
- **THEN** the system SHALL print a message that no CSV files were found and exit cleanly

### Requirement: Process files in chronological order
The system SHALL sort discovered CSV files alphabetically by filename before processing. Files SHALL be processed sequentially in sorted order so that the merchant cache builds up from earlier files to later ones.

#### Scenario: Alphabetical sort determines import order
- **WHEN** a directory contains `jan2026.csv`, `mar2026.csv`, `feb2026.csv`
- **THEN** the system SHALL process them in order: `feb2026.csv`, `jan2026.csv`, `mar2026.csv`

### Requirement: Per-file import summary
The system SHALL print a summary after each file is imported, showing: filename, total transactions parsed, new insertions, duplicates skipped, cache hits, and LLM calls.

#### Scenario: Summary printed after each file
- **WHEN** a file with 24 transactions is imported (20 new, 4 duplicates, 15 cache hits, 5 LLM calls)
- **THEN** the system SHALL print a summary line with those counts

### Requirement: Overall batch summary
After all files are processed, the system SHALL print an overall summary with totals across all files: total files processed, total transactions parsed, total new insertions, total duplicates skipped, total cache hits, total LLM calls.

#### Scenario: Overall summary after batch import
- **WHEN** 12 CSV files have been imported
- **THEN** the system SHALL print aggregated totals across all 12 files

### Requirement: Reusable import function
The per-file import logic SHALL be extracted into a reusable function that accepts a database reference, classifier reference, and CSV file path, and returns import statistics. Both single-file and batch modes SHALL use the same function.

#### Scenario: Single-file and batch share same logic
- **WHEN** importing a file via single-file mode or batch mode
- **THEN** the same underlying import function SHALL be used, producing identical results for the same input
