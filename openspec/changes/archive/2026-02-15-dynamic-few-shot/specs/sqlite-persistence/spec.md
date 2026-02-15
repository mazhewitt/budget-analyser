## ADDED Requirements

### Requirement: Few-shot examples CRUD
The Database SHALL provide methods to: (1) insert or replace a few-shot example by merchant_pattern, (2) load all few-shot examples, (3) delete LLM-sourced merchant cache entries by normalised key. The `few_shot_examples` table SHALL be created alongside the existing tables on startup.

#### Scenario: Insert few-shot example
- **WHEN** `insert_few_shot_example(merchant_pattern, raw_description, correct_merchant, correct_category)` is called
- **THEN** an example SHALL be stored, replacing any existing example with the same merchant_pattern

#### Scenario: Load all examples
- **WHEN** `get_few_shot_examples()` is called
- **THEN** all stored examples SHALL be returned

#### Scenario: Delete LLM cache entry
- **WHEN** `delete_llm_cache_entry(raw_key)` is called and the entry has source "llm"
- **THEN** the cache entry SHALL be deleted

#### Scenario: Preserve manual cache entry
- **WHEN** `delete_llm_cache_entry(raw_key)` is called and the entry has source "manual"
- **THEN** the cache entry SHALL NOT be deleted
