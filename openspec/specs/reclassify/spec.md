### Requirement: Reclassify subcommand
The system SHALL provide a `reclassify` CLI subcommand that re-runs LLM classification on flagged transactions using the current prompt (including dynamic few-shot examples). The command SHALL accept the same filter options as `review` (--threshold, --category, --since, --until, --merchant).

#### Scenario: Reclassify all flagged transactions
- **WHEN** the user runs `cargo run -- reclassify`
- **THEN** all flagged transactions SHALL be re-classified using the LLM with dynamic few-shot examples

#### Scenario: Reclassify with filters
- **WHEN** the user runs `cargo run -- reclassify --category Other`
- **THEN** only flagged transactions with category "Other" SHALL be re-classified

### Requirement: Reset before reclassification
Before re-classifying, the system SHALL reset each flagged transaction's category to "Uncategorised", confidence to 0.0, and source to "llm". LLM-sourced merchant cache entries for the affected merchants SHALL also be cleared so the LLM is invoked fresh. Manual cache entries (source = "manual") SHALL be preserved.

#### Scenario: LLM cache entries cleared
- **WHEN** reclassify runs on a transaction whose merchant cache entry has source "llm"
- **THEN** that cache entry SHALL be deleted before re-classification

#### Scenario: Manual cache entries preserved
- **WHEN** reclassify runs on a transaction whose merchant cache entry has source "manual"
- **THEN** that cache entry SHALL NOT be deleted (the manual correction takes precedence)

### Requirement: Reclassify summary
After reclassification, the system SHALL print a summary showing: total re-classified, cache hits (from manual entries), LLM calls, and category changes (how many moved to a different category).

#### Scenario: Summary after reclassify
- **WHEN** 10 transactions are reclassified (3 cache hits, 7 LLM calls, 5 changed category)
- **THEN** the summary SHALL show those counts
