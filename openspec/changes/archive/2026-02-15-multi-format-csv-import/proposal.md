## Why

The CSV parser only handles the synthetic UBS account statement format (comma-separated, flat headers). Real data arrives in two distinct formats — the UBS account statement export (semicolon-separated with metadata preamble, split description fields, ISO dates) and the UBS credit card invoice export (semicolon-separated, different column layout, no transaction IDs). The pipeline cannot import real data without adapting the parser to detect and handle each format.

## What Changes

- Auto-detect CSV format by sniffing file headers (account statement vs credit card vs synthetic)
- Parse UBS account statement: skip 8-line metadata preamble, semicolon delimiter, `yyyy-mm-dd` dates, concatenate Description1/2/3 fields
- Parse UBS credit card invoice: skip `sep=;` line, semicolon delimiter, `dd.mm.yyyy` dates, filter out non-transaction rows (balance brought forward, direct debit), generate stable transaction IDs from row content hash
- Existing synthetic format continues to work unchanged (backward compatible)
- Create realistic test fixture data for both new formats

## Capabilities

### New Capabilities
- `multi-format-detection`: Auto-detection of CSV format from file content, dispatching to the correct parser

### Modified Capabilities
- `csv-parsing`: Extended to handle three formats (synthetic, account statement, credit card invoice) instead of one. New struct fields or relaxed field requirements to accommodate formats that lack certain columns.

## Impact

- `src/csv_parser.rs` — primary change: format detection, three parsing paths
- No changes to downstream pipeline (`classifier.rs`, `db.rs`, `main.rs`) — all formats produce the same `Transaction` struct
- Test fixtures needed for account statement and credit card formats
