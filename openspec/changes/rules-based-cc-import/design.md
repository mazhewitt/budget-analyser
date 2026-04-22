## Context

The UBS credit card CSV export includes a `Sector` column populated from Visa/Mastercard Merchant Category Codes (MCC). Audit of the current 5,166 credit-card rows in the database (import batches `credit-card.csv` and `transactions-10.csv`) against their source CSVs shows:

- **106 unique non-empty sectors**; 62 map deterministically to a single budget category today; another ~20 are 95%+ consistent with the minority cases being LLM noise or clearly-wrong LLM overrides (e.g. `DIESEL LANDQUART` classified as Transport because the LLM pattern-matched "Diesel" to fuel, when it is a Diesel clothing store)
- **~10 sectors are genuinely mixed** (Hotels, Aparments, Business services, Banks) and benefit from merchant-string overrides
- **219 rows have empty Sector** — these always need an LLM classification
- Some current DB "majority" categories are themselves wrong (`Barber or beauty shops` → Housing, `Banks - merchandise and services` → Children without recipient context); the curated mapping SHALL correct these rather than mirror them

The CC CSV format is already detected in `src/csv_parser.rs` via `CsvFormat::CreditCard` (`sep=;` first-line marker). The current import path in `src/main.rs::import_file` is format-agnostic: it calls `classifier.classify` for every row regardless of format.

## Goals / Non-Goals

**Goals:**
- Deterministic classification for CC rows where we have signal (Sector or merchant pattern)
- Zero LLM calls for CC rows with known Sector or matching merchant override
- Correct classification for the three-kids TWINT pattern without reliance on LLM recall
- Consistent merchant-name normalisation (strip country/location suffixes, card masks)
- Clear reporting: import summaries SHALL distinguish rule hits from cache hits and LLM calls

**Non-Goals:**
- Not building a generic rules engine. This change ships a hand-coded Rust classifier. Migration to ZEN Engine / JDM is deferred.
- Not rewriting existing CC rows in the database. That is a separate follow-up.
- Not changing the account-statement or synthetic import paths. They continue to use the LLM classifier.
- Not introducing user-editable rules files. Rules live in Rust source for now.
- Not removing the merchant cache; it remains in place for the LLM path (account statements).

## Decisions

### 1. Routing: per-format classifier selection inside `import_file`

Add a thin dispatch at the top of the per-row loop:

```
if csv_format == CreditCard:
    result = cc_rules.classify(raw) or llm_fallback(row)
else:
    result = existing cache-then-LLM path
```

**Why not a trait-based polymorphic Classifier?** Only two concrete strategies exist and they have different inputs (rules needs the raw CSV row including Sector; LLM needs description/amount/details). A simple `match` on the format is clearer and avoids inventing an abstraction around a single branch.

**Alternatives considered:**
- Refactor `Classifier` into a trait with `RulesClassifier` and `LlmClassifier` impls — over-engineered for two strategies with asymmetric inputs.
- Pre-classify in `csv_parser` — couples parsing to classification; rules logic does not belong in the parser.

### 2. Exposing the CSV format to the import pipeline

`parse_csv` currently returns `Vec<Transaction>` and hides format. Two options:

- **(a)** Detect format a second time in `main::import_file` by re-opening the file's first line
- **(b)** Have `parse_csv` return `(CsvFormat, Vec<Transaction>)` (or add a `format` field to a new wrapper struct)

**Decision: (b)** — return the format alongside the transactions. One-pass, no double I/O, small refactor.

### 3. Rules module shape: `src/cc_rules.rs`

Pure functions, no state:

```rust
pub fn classify(row: &CreditCardRow) -> Option<ClassificationResult>
```

Returns `None` for "no confident rule match, please fall back". Internal structure:

1. `merchant_overrides() -> &'static [(Regex, &str, &str)]` — ordered list of `(pattern, merchant, category)`. First match wins. Highest priority. Examples:
   - `TWINT \*Sent to [KTL](?:\.?H[A-Z]*\.?)?\b` → ("TWINT to kid", "Children")
   - `TWINT \*Sent to ` (catch-all for other recipients) → ("TWINT P2P", "Transfers")
   - `TWINT \*UBS TWINT` → ("UBS TWINT top-up", "Transfers")
   - `(?i)^(MIGROS|COOP|ALDI|LIDL|DENNER)` → (normalised merchant, "Groceries") — regardless of sector
   - `(?i)UBS Rest\.` → ("UBS Staff Restaurant", "Dining") — overrides Hotels sector
2. `sector_to_category() -> &'static HashMap<&str, &str>` — the ~100-entry lookup. Categories drawn from the existing 18-category schema. See specs for the full table.
3. `normalise_merchant(booking_text) -> String` — strip trailing country codes (CHE/GBR/USA/IRL/DEU/NLD/ITA/FRA/SWE etc.), collapse repeated whitespace, strip trailing numeric location tails (`12345`, `ZH 2`), strip card masks (`XXXXXX`).

**Confidence values:**
- Merchant override hit → 0.95
- Sector lookup hit → 0.90
- No match → `None` (caller falls back to LLM)

**Why regex for overrides?** TWINT pattern needs anchored alternation; it is the one place a simple HashMap cannot express the rule. Small regex set (~10 patterns), compiled once with `once_cell::Lazy`.

### 4. Empty-sector handling

If `row.sector.is_empty()` AND no merchant override matches → return `None`, caller uses LLM. These are the 219 observed rows (mostly older exports or direct debits that slipped through the parser filter). The LLM fallback uses the existing cache, so this path continues to benefit from caching.

### 5. Source tracking

New value `source = "rules"` for rules-classified rows. Existing values (`cache`, `llm`) remain for non-CC paths. The `ImportStats` struct gains a `rules_hits: usize` field; summary lines gain a `rules hits` column.

### 6. Bypass the merchant cache on the rules path

Rules are deterministic and cheap; caching adds no value. More importantly, writing rules results into the cache would pollute it: the cache is currently keyed by normalised merchant and is used by non-CC rows too. We do NOT want a "MIGROS" cache entry written by the CC rules path to then be consumed by an account-statement row that happens to share the merchant normalisation.

### 7. Categories mapping: hand-curated, not auto-derived

The proposed `Sector → Category` table is curated by hand against the existing 18-category schema, NOT derived from the DB's current majority-wins distribution. Rationale: the DB contains known LLM errors (Barber=Housing, Theater=Dining, etc.). Majority-derivation would propagate these. The full table lives in specs and in `src/cc_rules.rs`; it is reviewed as code, not data.

## Risks / Trade-offs

- **Risk**: New UBS sector strings appear in future exports that are not in the lookup table → silently fall through to LLM. **Mitigation**: log a warning when a non-empty sector has no mapping entry, so new sectors surface immediately.
- **Risk**: Regex in the override list is a foot-gun (ordering matters, specificity matters). **Mitigation**: unit tests per-rule covering both positive and negative cases; an ordered list with comments explaining precedence.
- **Risk**: Hand-curated mapping encodes policy that may diverge from the user's intent over time. **Mitigation**: mapping lives in a single dedicated module; changes are a one-line code edit; `reclassify` command already exists for re-running classification over historical data.
- **Trade-off**: We drop the cache for CC rows, so cold imports of a new CC file cannot benefit from "I've seen this merchant before". Acceptable because rules path is already sub-millisecond per row.
- **Trade-off**: New sectors introduced by UBS require a source-code edit, not a config change. Documented as deferred until rule volume justifies ZEN Engine.
- **Trade-off**: Merchant-name normalisation is never perfect for free-text. We prefer under-normalising (leave a messy tail) over over-normalising (strip something meaningful) and lean on the override list for the merchants that matter.

## Open Questions

- Should we add an `Entertainment` category now (Cinema, Theater, Tourist attractions clearly want one) or force-fit them into `Other`? Current decision: force-fit into `Dining` or `Shopping` per majority, pending a future category-schema change. Can be revisited at implementation time.
- Confidence threshold: should rules-classified rows be flagged for review? Currently the review pipeline flags `confidence < 0.80`; rules produce 0.90/0.95, so they stay below review by default. No action needed.
