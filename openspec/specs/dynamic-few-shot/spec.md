### Requirement: Few-shot examples table schema
The system SHALL store few-shot examples with columns: `id` (INTEGER PRIMARY KEY), `merchant_pattern` (TEXT UNIQUE — normalised merchant key), `raw_description` (TEXT — the original transaction description), `correct_merchant` (TEXT), `correct_category` (TEXT), `created_at` (TEXT, ISO 8601). The `merchant_pattern` SHALL be unique so only the latest correction for a given merchant is kept.

#### Scenario: Store a new few-shot example
- **WHEN** a user corrects a transaction with raw description "BAZG VIA WEBSHOP" to category "Fees"
- **THEN** a few-shot example SHALL be stored with merchant_pattern = normalised key, raw_description = "BAZG VIA WEBSHOP", correct_merchant and correct_category from the correction

#### Scenario: Update existing example for same merchant
- **WHEN** a user re-corrects a merchant that already has a few-shot example
- **THEN** the existing example SHALL be replaced with the new correction

### Requirement: Load all few-shot examples for prompt injection
The system SHALL provide a method to load all few-shot examples from the database. The examples SHALL be formatted as additional prompt examples appended after the hardcoded examples in the system prompt.

#### Scenario: Examples loaded on import
- **WHEN** the import pipeline starts
- **THEN** all few-shot examples SHALL be loaded from the database and passed to the classifier

#### Scenario: Empty examples list
- **WHEN** no few-shot examples exist in the database
- **THEN** the classifier SHALL use only the hardcoded examples (no error)

### Requirement: Format examples for LLM prompt
Each few-shot example SHALL be formatted as a prompt line matching the hardcoded example format: `- "<raw_description>" → {{"merchant": "<correct_merchant>", "category": "<correct_category>", "confidence": 0.95}}`. These lines SHALL be appended after the hardcoded examples section.

#### Scenario: Dynamic example appears in prompt
- **WHEN** a few-shot example exists for "BAZG VIA WEBSHOP" → merchant "BAZG", category "Fees"
- **THEN** the system prompt SHALL include a line: `- "BAZG VIA WEBSHOP" → {"merchant": "BAZG", "category": "Fees", "confidence": 0.95}`
