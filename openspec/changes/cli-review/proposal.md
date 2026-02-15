## Why

After batch-importing a year of data, many transactions will have low confidence scores or be categorised as "Other" — the LLM's way of saying it isn't sure. These need human review to correct the merchant name and category. Without a review workflow, the user would have to write raw SQL updates against the database. A CLI review mode provides a structured way to surface flagged transactions, accept corrections, and feed those corrections back into the merchant cache.

## What Changes

- Add a `review` subcommand that lists transactions needing review (confidence < threshold or category = "Other" or "Uncategorised")
- For each flagged transaction, display the raw description, amount, date, details, and the LLM's classification
- Accept user input to: confirm the classification, change the category, or edit the merchant name
- When a correction is made, update both the transaction record and the merchant cache in SQLite
- Support optional filters: by category, by date range, by merchant substring
- Add a summary at the end showing how many were reviewed, confirmed, and corrected

## Capabilities

### New Capabilities
- `cli-review`: Interactive CLI workflow for reviewing and correcting flagged transactions, with filtering and correction persistence

### Modified Capabilities
- `sqlite-persistence`: Add query methods for fetching flagged transactions and updating transaction/cache records

## Impact

- **Code**: New `src/review.rs` module; new Database query/update methods in `db.rs`; update `main.rs` to add subcommand routing (import vs review)
- **CLI**: Move from positional args to subcommands: `cargo run -- import <path>` and `cargo run -- review [--category X] [--since YYYY-MM-DD] [--until YYYY-MM-DD]`
- **Dependencies**: None new — stdin reading uses `std::io`
