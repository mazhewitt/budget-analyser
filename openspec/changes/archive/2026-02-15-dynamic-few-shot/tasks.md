## Tasks

### 1. Add few_shot_examples table and Database methods
- [x] Add `CREATE TABLE IF NOT EXISTS few_shot_examples` to `Database::open()` with columns: id, merchant_pattern (UNIQUE), raw_description, correct_merchant, correct_category, created_at
- [x] Add `Database::insert_few_shot_example(merchant_pattern, raw_description, correct_merchant, correct_category)` using INSERT OR REPLACE
- [x] Add `Database::get_few_shot_examples()` returning all examples as a `Vec<FewShotExample>` struct
- [x] Add `Database::delete_llm_cache_entry(raw_key)` that deletes cache entries where source = "llm"

### 2. Define FewShotExample struct
- [x] Define a `FewShotExample` struct (merchant_pattern, raw_description, correct_merchant, correct_category) in `db.rs` or a shared location accessible by both `db.rs` and `classifier.rs`

### 3. Update classifier to accept dynamic examples
- [x] Change `Classifier::classify()` to accept `&[FewShotExample]` parameter
- [x] Change `build_system_prompt()` to accept `&[FewShotExample]` and append them as additional example lines after the hardcoded examples
- [x] Format each dynamic example as: `- "<raw_description>" → {{"merchant": "<correct_merchant>", "category": "<correct_category>", "confidence": 0.95}}`

### 4. Update review to store few-shot examples
- [x] On confirm (option 1): call `db.insert_few_shot_example()` with the normalised merchant pattern, raw description, current merchant, and current category
- [x] On category change (option 2): call `db.insert_few_shot_example()` with the corrected category
- [x] On merchant edit (option 3): call `db.insert_few_shot_example()` with the corrected merchant

### 5. Update import pipeline to use dynamic examples
- [x] In `import_file()`, load few-shot examples from DB once via `db.get_few_shot_examples()` and pass to each `classifier.classify()` call

### 6. Add reclassify subcommand
- [x] Add "reclassify" case to main.rs subcommand routing, with the same filter parsing as review (--threshold, --category, --since, --until, --merchant, db_path, model, endpoint)
- [x] Query flagged transactions using `db.get_flagged_transactions()`
- [x] For each flagged transaction: delete LLM cache entry for its normalised key, re-classify with LLM (passing dynamic examples), update the transaction, insert new cache entry
- [x] Track and print summary: total re-classified, cache hits (from manual entries), LLM calls, category changes

### 7. Verify end-to-end
- [x] Run `cargo build` — must compile without errors
- [x] Run import, then review and correct a transaction — verify few-shot example is stored in DB
- [x] Run reclassify — verify flagged transactions are re-classified using the improved prompt
- [x] Run import again with same CSV — verify cache still works correctly
