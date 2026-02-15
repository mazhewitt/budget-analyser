## MODIFIED Requirements

### Requirement: Classify transactions using local LLM with dynamic examples

The classifier SHALL accept an optional list of few-shot examples when building the system prompt. Dynamic examples SHALL be appended after the hardcoded examples in the prompt. The `classify()` method SHALL accept a `&[FewShotExample]` parameter. When the list is empty, behavior SHALL be identical to the current static prompt.

#### Scenario: Classification with dynamic examples
- **WHEN** the classifier is called with few-shot examples loaded from the database
- **THEN** the system prompt SHALL include both hardcoded and dynamic examples

#### Scenario: Classification without dynamic examples
- **WHEN** the classifier is called with an empty examples list
- **THEN** the system prompt SHALL contain only the hardcoded examples (backward compatible)
