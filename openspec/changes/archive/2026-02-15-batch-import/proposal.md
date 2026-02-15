## Why

The current CLI accepts a single CSV file per run. To populate the database with a full year of UBS exports (12 monthly files), the user must run the command 12 times manually. Batch import lets one command process all files chronologically, with the merchant cache building up across files so early imports train the cache and later imports benefit from it.

## What Changes

- Accept a directory path or multiple CSV file paths as CLI input instead of a single file
- Process CSV files in chronological order (sorted by filename or by first transaction date)
- Print a per-file summary after each import (transactions, cache hits, LLM calls, duplicates)
- Print an overall summary across all files at the end
- Extract the single-file import logic into a reusable function so both single-file and batch modes share the same pipeline

## Capabilities

### New Capabilities
- `batch-import`: Accept multiple CSV files, process them in order, accumulate stats, and print per-file and overall summaries

### Modified Capabilities
None — the existing import pipeline (`sqlite-persistence`, `duplicate-detection`, `csv-parsing`) is reused as-is. This change is purely orchestration.

## Impact

- **Code**: Refactor `main.rs` to extract import logic into a function; add file discovery/sorting logic
- **CLI**: Change from single positional arg to directory or glob pattern (e.g., `cargo run -- data/exports/ data/budget.db`)
- **Dependencies**: None new — uses `std::fs` for directory listing
