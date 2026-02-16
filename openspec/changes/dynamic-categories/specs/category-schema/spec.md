## MODIFIED Requirements

### Requirement: Define transaction category taxonomy

The system SHALL source the set of spending categories from the `categories` database table at runtime, rather than from a hardcoded Rust enum. The default categories SHALL be seeded from the enum on first run, but new categories added to the database SHALL be immediately available for classification and review. Each category SHALL have a name and a description suitable for inclusion in an LLM prompt. The Uncategorised category SHALL always be present as a fallback for unclassifiable transactions.

#### Scenario: All categories are representable

- **WHEN** a transaction is classified
- **THEN** the result SHALL be one of the category names present in the `categories` table

#### Scenario: Uncategorised as fallback

- **WHEN** a transaction cannot be confidently classified into any specific category
- **THEN** it SHALL be assigned the "Uncategorised" category

#### Scenario: Newly added categories are available for classification

- **WHEN** a new category has been added to the `categories` table
- **THEN** the classifier SHALL include it in the LLM prompt and accept it as a valid classification result

### Requirement: Category schema is embeddable in LLM prompts

The system SHALL generate a text block listing all categories (names + descriptions) for inclusion in an LLM system prompt by reading from the `categories` database table. The output SHALL list each category with its description so the model understands what each category covers.

#### Scenario: Generate prompt-ready schema text

- **WHEN** the schema is serialised for prompt inclusion
- **THEN** the output SHALL contain every category name from the database paired with its description, one per line

### Requirement: Category selection in review uses database categories

The review flow SHALL present all categories from the `categories` table when the user selects option (2) to change a category. The menu SHALL be numbered and include all database-sourced categories.

#### Scenario: Review shows dynamic categories

- **WHEN** the user chooses to change category during review
- **THEN** the system SHALL display all categories from the database as numbered options
