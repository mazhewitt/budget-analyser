## MODIFIED Requirements

### Requirement: Define transaction category taxonomy

The system SHALL define a fixed set of spending categories as a Rust enum. The categories SHALL include: Groceries, Dining, Transport, Housing, Insurance, Healthcare, Shopping, Subscriptions, Children, Travel, Cash, Transfers, Income, Fees, and Other. Each category SHALL have a human-readable display name and a brief description suitable for inclusion in an LLM prompt. The Uncategorised variant SHALL be retained as a fallback for unclassifiable transactions.

#### Scenario: All categories are representable

- **WHEN** a transaction is classified
- **THEN** the result SHALL be one of the 16 defined category enum variants (15 categories + Uncategorised)

#### Scenario: Uncategorised as fallback

- **WHEN** a transaction cannot be confidently classified into any specific category
- **THEN** it SHALL be assigned the Uncategorised category

## REMOVED Requirements

### Requirement: POC-only categories
**Reason**: The POC categories Utilities, Taxes, and CardPayments are replaced by the expanded schema. Utilities is absorbed into Housing (for home utilities) and Subscriptions (for phone/internet). Taxes is absorbed into Fees. CardPayments is absorbed into Transfers.
**Migration**: Update any existing data to map old categories to new ones. No existing production data exists yet, so this is a clean swap.
