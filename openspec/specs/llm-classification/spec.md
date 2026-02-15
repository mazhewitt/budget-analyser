## ADDED Requirements

### Requirement: Classify transactions using local LLM with dynamic examples

The classifier SHALL accept an optional list of few-shot examples when building the system prompt. Dynamic examples SHALL be appended after the hardcoded examples in the prompt. The `classify()` method SHALL accept a `&[FewShotExample]` parameter. When the list is empty, behavior SHALL be identical to the current static prompt.

#### Scenario: Classification with dynamic examples
- **WHEN** the classifier is called with few-shot examples loaded from the database
- **THEN** the system prompt SHALL include both hardcoded and dynamic examples

#### Scenario: Classification without dynamic examples
- **WHEN** the classifier is called with an empty examples list
- **THEN** the system prompt SHALL contain only the hardcoded examples (backward compatible)

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
