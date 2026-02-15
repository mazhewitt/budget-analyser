## Why

When the LLM misclassifies a merchant, the user corrects it during review. Currently that correction only updates the transaction record and merchant cache — it doesn't improve future LLM calls for similar but not identical merchants. Dynamic few-shot learning stores corrections as examples that are injected into the LLM prompt, so the model learns from past mistakes. Over time, the prompt becomes increasingly tailored to this user's actual spending patterns. A `reclassify` command lets the user re-run the LLM on flagged transactions with the improved prompt.

## What Changes

- Add a `few_shot_examples` table to SQLite for storing user corrections as prompt examples
- When a user corrects a transaction in review, store the correction as a few-shot example keyed by normalised merchant pattern
- Update the classifier's `build_system_prompt()` to accept dynamic examples loaded from the database and append them to the hardcoded examples
- Add a `reclassify` CLI subcommand that re-runs the LLM on flagged transactions using the improved prompt (clears old classification, re-classifies with dynamic examples)
- During import, load few-shot examples from the database so new CSVs benefit from past corrections

## Capabilities

### New Capabilities
- `dynamic-few-shot`: Store, retrieve, and inject user corrections as few-shot examples into LLM classification prompts
- `reclassify`: Re-run LLM classification on flagged transactions using the improved prompt with dynamic examples

### Modified Capabilities
- `llm-classification`: Update the prompt builder to accept and inject dynamic few-shot examples
- `sqlite-persistence`: Add the `few_shot_examples` table and CRUD methods
- `cli-review`: Store a few-shot example when the user makes a correction

## Impact

- **Code**: New methods in `db.rs` for few-shot CRUD; modify `classifier.rs` to accept dynamic examples; modify `review.rs` to store examples on correction; add `reclassify` subcommand to `main.rs`
- **Schema**: New `few_shot_examples` table in SQLite
- **CLI**: New `reclassify` subcommand
- **Dependencies**: None new
