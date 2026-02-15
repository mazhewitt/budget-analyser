## MODIFIED Requirements

### Requirement: Accept user corrections

For each flagged transaction, the system SHALL present options: (1) Confirm, (2) Change category, (3) Edit merchant, (4) Skip, (5) Quit. When the user confirms or corrects a transaction, the system SHALL store the correction as a few-shot example in addition to updating the transaction and merchant cache.

#### Scenario: Correction stored as few-shot example
- **WHEN** the user corrects a transaction's category from "Other" to "Fees"
- **THEN** a few-shot example SHALL be stored with the normalised merchant pattern, raw description, corrected merchant name, and corrected category

#### Scenario: Confirm stores few-shot example
- **WHEN** the user confirms an LLM classification
- **THEN** a few-shot example SHALL be stored with the confirmed merchant and category
