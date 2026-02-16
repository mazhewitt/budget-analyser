## Context

The `review` command queries flagged transactions (low confidence OR Other/Uncategorised), groups by normalised merchant key, and presents an interactive menu per group. The `recategorise` feature needs the same group-and-prompt pattern but querying by an exact category instead of the flagged-transaction filter.

The review flow already handles: grouping by merchant, displaying transaction details, category selection via numbered menu, updating transactions + cache + few-shot examples. We should reuse as much of this as possible.

## Goals / Non-Goals

**Goals:**
- Query all transactions in a specific category regardless of confidence
- Present grouped interactive review with the same UX as the existing review flow
- Update cache and few-shot examples so future imports benefit

**Non-Goals:**
- Bulk "move all from X to Y" without review — too dangerous, user should see what's moving
- LLM-assisted proposals — future enhancement, not this change
- Changing the review flow itself — this is additive

## Decisions

### 1. New DB method rather than modifying `get_flagged_transactions()`

Add `get_transactions_by_category(category)` as a separate method rather than adding more conditional logic to `get_flagged_transactions()`. The flagged query has a specific WHERE clause (confidence < threshold OR Other/Uncategorised) that's fundamentally different from "give me everything in category X".

**Why**: Cleaner separation. The flagged query is already complex with optional filters. Mixing in a "bypass the confidence filter" mode would make it harder to reason about.

### 2. New `run_recategorise()` function in review.rs

Add a dedicated function rather than adding flags to `run_review()`. The interaction loop is nearly identical but the data source and framing differ (reviewing flagged items vs deliberately reassigning a category).

**Why**: The functions share the group-and-prompt pattern, but the entry message, menu options, and stats framing are different enough to warrant separation. The shared logic (category selection, DB updates) is already straightforward enough that extracting shared helpers would be premature.

### 3. Simplified menu — no "Confirm" option

The recategorise menu should be: (1) Change category, (2) Skip, (3) Quit. There's no "Confirm" option because the user is explicitly here to *change* things — confirming the current category doesn't make sense in this context.

**Why**: The user triggered recategorise because they want to move transactions out. Keeping them where they are is "Skip", not "Confirm".

## Risks / Trade-offs

- **Large categories could be tedious** → Mitigation: the group-by-merchant reduces the number of interactions. A category with 200 transactions from 5 merchants is only 5 prompts.
- **No undo** → Same as existing review flow. The cache and few-shot updates make this sticky. Acceptable for now.
