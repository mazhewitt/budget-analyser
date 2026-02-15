## Tasks

### 1. Create ImportStats struct
- [x] Define an `ImportStats` struct with fields: `total_parsed`, `new_insertions`, `duplicates_skipped`, `cache_hits`, `llm_calls`
- [x] Implement `Default` and an `accumulate(&mut self, other: &ImportStats)` method for summing across files

### 2. Extract import_file function
- [x] Extract the per-file import loop from `main.rs` into `fn import_file(db: &Database, classifier: &Classifier, csv_path: &Path) -> Result<ImportStats>`
- [x] The function should parse the CSV, classify each transaction (cache or LLM), insert into DB, log the import, and return stats
- [x] Print per-transaction progress lines from within the function

### 3. Add file/directory detection in main
- [x] Detect whether the first CLI arg is a file or directory using `std::fs::metadata`
- [x] If file: call `import_file()` once (backward compatible)
- [x] If directory: collect all `*.csv` files, sort alphabetically, call `import_file()` for each

### 4. Add per-file and overall summaries
- [x] After each `import_file()` call, print a per-file summary with the stats
- [x] After all files, print an overall summary with accumulated totals across all files
- [x] Include total files processed count in the overall summary

### 5. Verify end-to-end
- [x] Run `cargo build` — must compile without errors
- [x] Run with a single CSV file — behavior should be identical to before
- [x] Run with a directory containing the synthetic CSV — should discover and import it
- [x] Run again with same directory — all transactions should be duplicates
