## ADDED Requirements

### Requirement: Normalise merchant strings for cache keying

The system SHALL normalise transaction description strings by: (1) converting to uppercase, (2) stripping known suffixes ("EBILL-RECHT", "EBILL", "KARTE" followed by digits), (3) removing trailing digit-only tokens (reference numbers, transaction fragments), and (4) collapsing multiple whitespace into single spaces and trimming. The normalised string SHALL be used as the cache lookup key.

#### Scenario: Group SBB MOBILE variants

- **WHEN** descriptions "SBB MOBILE" from different transactions are normalised
- **THEN** they SHALL all produce the same cache key

#### Scenario: Strip EBILL suffix

- **WHEN** description "Steuerverwaltung EBILL-RECHT" is normalised
- **THEN** the cache key SHALL be "STEUERVERWALTUNG"

#### Scenario: Keep distinct merchants separate

- **WHEN** descriptions "COOP PRONTO BHFSTR ZURICH" and "COOP CITY BASEL" are normalised
- **THEN** they SHALL produce different cache keys

#### Scenario: Strip trailing reference numbers

- **WHEN** description "MIGROS BASEL M 01234 KARTE 1234" is normalised
- **THEN** trailing digit tokens and KARTE suffix SHALL be removed, producing a key like "MIGROS BASEL M"

### Requirement: Cache classification results in SQLite

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
