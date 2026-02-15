## Context

After `sqlite-storage`, the system can import a single CSV file into SQLite with duplicate detection and merchant caching. The import logic lives entirely in `main.rs` as inline code. To support batch import of a year of monthly exports, we need to extract the per-file import logic and add orchestration for multiple files.

UBS monthly exports follow a predictable naming pattern (e.g., `ubstransactions-jan2026.csv`, `ubstransactions-feb2026.csv`). Sorting by filename gives chronological order, which is important because the merchant cache should build up from older files to newer ones.

## Goals / Non-Goals

**Goals:**
- Import all CSV files from a directory in one command
- Process files chronologically so the cache warms up over time
- Show per-file and overall summaries
- Keep single-file import working (backward compatible)

**Non-Goals:**
- Parallel import (sequential is fine — LLM calls are the bottleneck anyway)
- Recursive directory scanning (flat directory only)
- Any changes to the classification or storage logic

## Decisions

### CLI interface: first arg is a path (file or directory)
If the first argument is a file, import that single file (backward compatible). If it's a directory, discover all `*.csv` files in it, sort alphabetically, and import each in order. This is the simplest approach — no new flags, no glob parsing, just path detection via `std::fs::metadata`. Alternative: require explicit `--batch` flag — rejected as unnecessary complexity.

### Extract `import_file()` function
Move the per-file import loop from `main.rs` into a standalone function `import_file(db, classifier, csv_path) -> ImportStats`. This function returns a stats struct with counts (parsed, inserted, duplicates, cache_hits, llm_calls). The main function calls it once for single-file mode or in a loop for batch mode.

### Sort files alphabetically for chronological ordering
UBS export filenames contain the month/year. Alphabetical sort gives chronological order for consistent naming patterns. If filenames don't sort chronologically, the user can rename them. Alternative: parse dates from file contents — rejected as over-engineering for a single-user tool.

## Risks / Trade-offs

- **[Risk] Non-chronological filenames** → User's responsibility to name files consistently. The system sorts alphabetically and processes in that order.
- **[Trade-off] No progress bar** → Simple println per file is sufficient for 12 files. If needed later, can add a progress indicator.
