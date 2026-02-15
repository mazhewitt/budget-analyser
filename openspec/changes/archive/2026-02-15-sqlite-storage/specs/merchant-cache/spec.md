## MODIFIED Requirements

### Requirement: Cache classification results in memory

The system SHALL maintain a merchant cache backed by SQLite, mapping normalised merchant keys to their classification result (merchant name, category, confidence, source). On startup, the cache SHALL be available immediately via database queries. When a transaction's normalised key matches a cached entry, the cached result SHALL be returned without calling the LLM.

#### Scenario: Cache hit avoids LLM call

- **WHEN** a transaction with normalised key "SBB MOBILE" is classified and the merchant_cache table already contains an entry for "SBB MOBILE"
- **THEN** the cached result SHALL be returned and no Ollama API call SHALL be made

#### Scenario: Cache miss triggers LLM call

- **WHEN** a transaction's normalised key has no cache entry
- **THEN** the system SHALL call the LLM, store the result in the merchant_cache table, and return it

#### Scenario: Cache persists across runs

- **WHEN** the application is restarted after previous classifications
- **THEN** all previously cached merchant mappings SHALL be available for lookup without re-calling the LLM
