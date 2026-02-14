## ADDED Requirements

### Requirement: Define transaction category taxonomy

The system SHALL define a fixed set of spending categories as a Rust enum. The categories SHALL include: Groceries, Transport, Utilities, Subscriptions, Transfers, CardPayments, Insurance, Taxes, Shopping, Dining, Income, and Uncategorised. Each category SHALL have a human-readable display name and a brief description suitable for inclusion in an LLM prompt.

#### Scenario: All categories are representable

- **WHEN** a transaction is classified
- **THEN** the result SHALL be one of the defined category enum variants

#### Scenario: Uncategorised as fallback

- **WHEN** a transaction cannot be confidently classified into any specific category
- **THEN** it SHALL be assigned the Uncategorised category

### Requirement: Category schema is embeddable in LLM prompts

The system SHALL provide a method to serialise the full category schema (names + descriptions) into a text block suitable for inclusion in an LLM system prompt. The output SHALL list each category with its description so the model understands what each category covers.

#### Scenario: Generate prompt-ready schema text

- **WHEN** the schema is serialised for prompt inclusion
- **THEN** the output SHALL contain every category name paired with its description, one per line
