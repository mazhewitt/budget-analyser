## Context

After importing a year of transactions via `batch-import`, the database contains classified transactions. Many will have low confidence or fall into "Other"/"Uncategorised". The user needs a way to review these, correct them, and have corrections persist in both the transaction record and the merchant cache (so future imports of the same merchant benefit).

Currently `main.rs` only does import — there's no subcommand structure. The CLI uses positional args. Adding review requires introducing subcommands.

## Goals / Non-Goals

**Goals:**
- Interactive CLI review of flagged transactions
- Corrections update both the transaction and the merchant cache
- Filters for targeted review (category, date range, merchant)
- Subcommand structure for import and review

**Non-Goals:**
- Few-shot examples (next change: `dynamic-few-shot`)
- Re-classification / reclassify command (next change)
- Web UI for review (Phase 2)
- Undo/rollback of corrections

## Decisions

### CLI argument parsing: manual subcommand routing
Use a simple manual match on the first argument (`import` / `review`) rather than adding a CLI parsing crate like `clap`. The subcommands are simple enough that hand-parsing is fine. Backward compatibility: if the first arg is a file/directory path (not a subcommand keyword), treat it as `import` for backward compatibility. Alternative: `clap` — rejected as unnecessary dependency for 2 subcommands.

### Review module: `src/review.rs`
Encapsulate the review loop in a dedicated module. The module takes a `&Database` reference and filter parameters, queries flagged transactions, and runs the interactive loop. This keeps `main.rs` focused on routing.

### Interactive flow: numbered menu per transaction
For each flagged transaction, show details and present options:
1. **Confirm** — accept the LLM's classification as-is (updates confidence to 1.0, source to "manual")
2. **Change category** — show numbered list of categories, user picks one
3. **Edit merchant** — user types a new merchant name
4. **Skip** — move to next without changing anything
5. **Quit** — exit review early

After confirm/change/edit, the transaction and merchant cache are updated immediately.

### Flagging criteria
A transaction is "flagged" if:
- `confidence < threshold` (default 0.80, configurable), OR
- `category = 'Other'`, OR
- `category = 'Uncategorised'`

### Filter parameters
- `--category <name>`: only show transactions in this category
- `--since <YYYY-MM-DD>`: only show transactions on or after this date
- `--until <YYYY-MM-DD>`: only show transactions on or before this date
- `--merchant <substring>`: only show transactions whose merchant name contains this substring
- `--threshold <float>`: override the confidence threshold (default 0.80)

Filters are combined with AND logic.

## Risks / Trade-offs

- **[Risk] Large review queues** → Filters help scope the review. Can also quit and resume later since corrections persist immediately.
- **[Trade-off] No batch confirm** → Reviewing one-by-one is slower but more accurate. Batch operations can be added later if needed.
- **[Trade-off] Manual subcommand parsing** → Simpler than clap but less robust error messages. Acceptable for a single-user tool.
