## Why

UBS credit card CSVs already contain a `Sector` column (MCC classification from Visa/Mastercard) that strongly determines the correct budget category. The current pipeline discards this signal: it passes the sector as free text to the LLM, which produces inconsistent results — the same merchant is classified differently on different runs, and a handful of TWINT P2P patterns (transfers to the user's three children) scatter across unrelated categories. A rules-based classifier that consumes the Sector directly would be deterministic, faster, cheaper, and more accurate for the 5,000+ credit-card-row majority of the corpus, while the LLM remains essential for the account-statement CSVs that have no sector column.

## What Changes

- Introduce a rules-based classifier that maps a credit card row to `(merchant, category, confidence, source)` using:
  - A hand-curated `Sector → Category` table covering the ~100 MCC sectors observed in UBS data
  - An ordered list of merchant-string overrides (e.g. `TWINT *Sent to [KTL]H...` → Children) evaluated BEFORE the sector lookup
  - Deterministic merchant-name normalisation (trim trailing country codes, location codes, card masks)
- Wire the rules classifier into the import pipeline for credit card format rows only (detected via the existing `CsvFormat::CreditCard` enum variant). Account statement and synthetic formats SHALL continue to use the LLM classifier unchanged.
- Rules path SHALL short-circuit the existing merchant cache lookup for credit card rows: deterministic rules make the cache redundant for this path and avoid polluting the cache with per-merchant entries that should be policy-driven.
- When the rules classifier cannot produce a confident result (empty sector AND no merchant override match), SHALL fall back to the existing LLM classifier.
- Report a new `rules_hits` count alongside `cache_hits` and `llm_calls` in per-file and overall import summaries.
- Scope: new CC imports only. Existing CC rows in the database are NOT rewritten by this change. Reclassification of the 5,000+ existing rows can be done later via the existing `reclassify` command or a follow-up change.

## Capabilities

### New Capabilities
- `rules-based-cc-classification`: Deterministic merchant & category assignment for UBS credit card CSV rows, driven by an MCC/Sector lookup table and an ordered merchant-override list, with documented fallback to LLM classification for unmatched rows.

### Modified Capabilities
- `batch-import`: Per-file and overall summaries SHALL include a `rules_hits` count. Credit-card-format rows SHALL route through the rules classifier before the cache/LLM path.

## Impact

- **Code**: new module `src/cc_rules.rs` (classifier + mapping tables). Modifications in `src/main.rs` (import pipeline routing), `src/csv_parser.rs` (expose CSV format on parsed transactions OR re-detect per file in main), and the `ImportStats` struct (new counter).
- **No new crate dependencies.** Hand-coded Rust only; ZEN Engine / Rust-Rule-Engine deferred until rule volume justifies it.
- **Database**: no schema change. Transactions inserted via the rules path SHALL use `source = "rules"` (new source string alongside existing `cache`, `llm`).
- **Performance**: credit card imports SHALL complete without LLM calls for rows with a known sector, reducing import time from minutes to seconds for typical statement batches.
- **Behavioural change**: import results for credit card files SHALL differ from the current LLM-driven output. Notable fixes include: `TWINT *Sent to {K,T,L}.H...` → Children (currently scattered); `Migros*`, `Coop*`, `Aldi`, `Lidl` → Groceries regardless of sector; `UBS Rest.` / cafeteria chains → Dining even when MCC says Hotels.
