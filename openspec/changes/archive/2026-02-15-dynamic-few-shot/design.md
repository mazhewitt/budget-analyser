## Context

The classifier uses a static system prompt with hardcoded few-shot examples. When users correct misclassifications in the review workflow, corrections persist to the transaction and merchant cache — but they don't improve the LLM's behavior for new, unseen merchants. Dynamic few-shot learning bridges this gap by storing corrections as examples that are injected into the LLM prompt.

The current `build_system_prompt()` in `classifier.rs` is a static method. It needs to become dynamic, accepting examples loaded from the database.

## Goals / Non-Goals

**Goals:**
- Store user corrections as few-shot examples in SQLite
- Inject relevant examples into the LLM classification prompt
- Add a reclassify command to re-process flagged transactions
- Improve classification accuracy over time as corrections accumulate

**Non-Goals:**
- Semantic similarity matching for examples (use exact merchant pattern match)
- Limiting the number of examples in the prompt (keep it simple — the prompt won't grow beyond a few dozen examples in practice)
- Removing hardcoded examples (they remain as the baseline)

## Decisions

### Few-shot example keying: normalised merchant pattern
Examples are keyed by the normalised merchant key (same as the cache). When building the prompt, load ALL examples from the database and append them after the hardcoded examples. We don't filter by relevance — the full set is small enough (tens, not thousands) to include in every prompt. Alternative: fuzzy matching or embedding-based retrieval — rejected as over-engineering for the expected volume.

### Classifier API change: pass examples to classify()
Change `Classifier::classify()` to accept a `&[FewShotExample]` parameter. The caller (import loop, reclassify) loads examples from the database once and passes them to each classify call. This avoids the classifier depending on the database directly. `build_system_prompt()` becomes `build_system_prompt(examples: &[FewShotExample])`.

### Reclassify: reset and re-run flagged transactions
The `reclassify` command queries flagged transactions (same criteria as review), clears their classification (resets to Uncategorised/0.0), then re-runs the LLM with the current prompt (including dynamic examples). It also clears the merchant cache entries that were LLM-sourced, so cache hits don't bypass the improved prompt. Manual corrections and their cache entries are preserved.

### Store example on every correction (not just new merchants)
Whenever the user confirms or corrects a transaction in review, store a few-shot example. This means confirming an LLM guess also teaches the model. Duplicate examples for the same merchant pattern are fine — INSERT OR REPLACE on the merchant_pattern key keeps only the latest.

## Risks / Trade-offs

- **[Risk] Prompt length growth** → Acceptable for tens of examples. If hundreds accumulate, add a limit later. The 8B model can handle several thousand tokens of examples.
- **[Trade-off] All examples in every prompt** → Simple but includes irrelevant examples. Acceptable because volume is small and the model can handle noise.
- **[Trade-off] Reclassify clears LLM cache entries** → Necessary to force re-evaluation, but means the next import after reclassify will hit the LLM for previously cached merchants. Corrections that were manually confirmed are preserved.
