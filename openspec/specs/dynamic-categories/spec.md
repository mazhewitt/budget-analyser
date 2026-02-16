# Capability: dynamic-categories

Runtime management of spending categories — storing category definitions in the database, listing available categories, and adding new ones without code changes.

## Requirements

### Requirement: Categories are stored in the database

The system SHALL store category definitions in a `categories` table with columns `name` (text, primary key) and `description` (text). On database initialisation, if the table is empty, the system SHALL seed it with the 17 default categories (Groceries, Dining, Transport, Housing, Insurance, Healthcare, Shopping, Subscriptions, Children, Travel, Cash, Transfers, Income, Investments, Fees, Other, Uncategorised) and their descriptions.

#### Scenario: First run seeds default categories

- **WHEN** the database is opened for the first time
- **THEN** the `categories` table SHALL contain exactly 17 rows matching the current hardcoded defaults

#### Scenario: Subsequent opens do not duplicate

- **WHEN** the database is opened and the `categories` table already contains rows
- **THEN** no additional rows SHALL be inserted by the seeding logic

### Requirement: List all available categories

The system SHALL provide a `list_categories()` function on the `Database` struct that returns all category definitions (name and description) ordered by name.

#### Scenario: List categories returns all entries

- **WHEN** `list_categories()` is called
- **THEN** it SHALL return all rows from the `categories` table as `CategoryInfo` structs with `name` and `description` fields

### Requirement: Add a new category

The system SHALL provide an `add_category(name, description)` function on the `Database` struct that inserts a new category into the `categories` table. The function SHALL reject duplicate names with an error.

#### Scenario: Add a unique category

- **WHEN** `add_category("Pets", "Pet food, vet bills, grooming")` is called
- **THEN** the `categories` table SHALL contain a new row with name "Pets" and the given description

#### Scenario: Reject duplicate category name

- **WHEN** `add_category("Groceries", "...")` is called and "Groceries" already exists
- **THEN** the function SHALL return an error indicating the category already exists

### Requirement: CLI subcommand to list categories

The system SHALL accept `categories list` as a CLI subcommand that prints all available categories with their descriptions.

#### Scenario: List categories via CLI

- **WHEN** the user runs `budget-analyser categories list`
- **THEN** the system SHALL print each category name and description, one per line

### Requirement: CLI subcommand to add a category

The system SHALL accept `categories add <name> <description>` as a CLI subcommand that adds a new category to the database.

#### Scenario: Add category via CLI

- **WHEN** the user runs `budget-analyser categories add "Pets" "Pet food, vet bills, grooming"`
- **THEN** the system SHALL insert the category and print a confirmation message

#### Scenario: Add duplicate via CLI

- **WHEN** the user runs `budget-analyser categories add "Groceries" "..."`
- **THEN** the system SHALL print an error message indicating the category already exists

### Requirement: Uncategorised always exists

The system SHALL ensure the "Uncategorised" category is always present in the `categories` table. It SHALL be included in the default seed set and cannot be removed.

#### Scenario: Uncategorised is present after seeding

- **WHEN** the database is initialised
- **THEN** the `categories` table SHALL contain an "Uncategorised" row
