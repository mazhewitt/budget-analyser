## ADDED Requirements

### Requirement: Query all transactions by category

The system SHALL provide a `get_transactions_by_category(category)` method on the `Database` struct that returns all transactions matching the given category name, regardless of confidence score, ordered by date ascending.

#### Scenario: Retrieve transactions for a specific category

- **WHEN** `get_transactions_by_category("Transfers")` is called
- **THEN** it SHALL return all transactions where `category = 'Transfers'`, ordered by date ascending

#### Scenario: No transactions in category

- **WHEN** `get_transactions_by_category("Nonexistent")` is called and no transactions have that category
- **THEN** it SHALL return an empty list

### Requirement: Interactive recategorise by merchant group

The system SHALL provide a `run_recategorise()` function that groups all transactions in a given category by normalised merchant key and presents each group to the user with options to change category or skip.

#### Scenario: Group display

- **WHEN** a merchant group is presented during recategorise
- **THEN** the system SHALL display the merchant name, current category, transaction count, total amount, and individual transaction details

#### Scenario: Change category for a group

- **WHEN** the user selects "Change category" for a merchant group
- **THEN** the system SHALL display all available categories as a numbered list, accept the user's selection, and update all transactions in the group plus the merchant cache and few-shot examples

#### Scenario: Skip a group

- **WHEN** the user selects "Skip" for a merchant group
- **THEN** the system SHALL leave those transactions unchanged and move to the next group

#### Scenario: Quit early

- **WHEN** the user selects "Quit" during recategorise
- **THEN** the system SHALL print a summary of changes made and exit

### Requirement: CLI recategorise subcommand

The system SHALL accept `recategorise --category <name>` as a CLI subcommand that triggers the interactive recategorise flow for the specified category.

#### Scenario: Recategorise a category

- **WHEN** the user runs `budget-analyser recategorise --category Transfers`
- **THEN** the system SHALL load all Transfers transactions, group by merchant, and start the interactive recategorise flow

#### Scenario: No category flag provided

- **WHEN** the user runs `budget-analyser recategorise` without `--category`
- **THEN** the system SHALL print a usage message indicating `--category` is required

#### Scenario: Category has no transactions

- **WHEN** the user runs `budget-analyser recategorise --category Empty`
- **THEN** the system SHALL print "No transactions found in category 'Empty'."
