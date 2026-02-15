## ADDED Requirements

### Requirement: Classify transactions via local Ollama model

The system SHALL send each transaction's description (and optionally amount and details) to a local Ollama instance via `POST /api/chat` with JSON mode enabled. The system SHALL construct a system prompt containing the category schema and few-shot examples of UBS merchant strings. The LLM SHALL return a JSON object with fields: merchant (normalised merchant name), category (one of the defined categories), and confidence (float 0.0–1.0).

#### Scenario: Successful classification of clear merchant

- **WHEN** a transaction with description "SBB MOBILE" is sent to the LLM
- **THEN** the response SHALL contain a valid merchant name, a category from the defined schema, and a confidence score between 0.0 and 1.0

#### Scenario: Classification of cryptic merchant string

- **WHEN** a transaction with description "Steuerverwaltung EBILL-RECHT" is sent to the LLM
- **THEN** the response SHALL still return a valid JSON with merchant, category, and confidence fields

#### Scenario: Invalid JSON response from LLM

- **WHEN** the LLM returns a response that cannot be parsed as the expected JSON schema
- **THEN** the system SHALL treat it as a classification failure and assign Uncategorised with confidence 0.0

### Requirement: Support configurable model selection

The system SHALL accept a model name parameter (e.g., "llama3.1:8b", "qwen2.5:7b") and pass it in the Ollama API request. This allows comparing classification accuracy across models.

#### Scenario: Switch between models

- **WHEN** the system is configured with model "qwen2.5:7b"
- **THEN** the Ollama API request SHALL specify `"model": "qwen2.5:7b"`

### Requirement: Configurable Ollama endpoint

The system SHALL accept a base URL for the Ollama API (defaulting to `http://localhost:11434`).

#### Scenario: Default endpoint

- **WHEN** no custom endpoint is provided
- **THEN** the system SHALL connect to `http://localhost:11434`

#### Scenario: Custom endpoint

- **WHEN** the endpoint is set to `http://192.168.1.10:11434`
- **THEN** the system SHALL use that URL for API requests
