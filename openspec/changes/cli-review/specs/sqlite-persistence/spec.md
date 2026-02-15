## ADDED Requirements

### Requirement: Query flagged transactions
The Database SHALL provide a method to query transactions that need review, accepting optional filter parameters (category, date range, merchant substring, confidence threshold). The method SHALL return a list of transaction records matching the flagging criteria AND any applied filters.

#### Scenario: Query with default threshold
- **WHEN** `get_flagged_transactions(threshold=0.80)` is called with no filters
- **THEN** all transactions with confidence < 0.80 OR category in ("Other", "Uncategorised") SHALL be returned

#### Scenario: Query with category filter
- **WHEN** `get_flagged_transactions(threshold=0.80, category="Other")` is called
- **THEN** only transactions with category "Other" that also meet the flagging criteria SHALL be returned

### Requirement: Update transaction classification
The Database SHALL provide a method to update a transaction's merchant_name, category, confidence, and source fields by transaction id. This is used when the user corrects a classification during review.

#### Scenario: Update transaction after correction
- **WHEN** `update_transaction(id, merchant="SBB", category="Transport", confidence=1.0, source="manual")` is called
- **THEN** the transaction record SHALL be updated with the new values

### Requirement: Update merchant cache entry
The Database SHALL provide a method to update or insert a merchant cache entry by normalised key, setting the merchant name, category, confidence, and source. The `updated_at` timestamp SHALL be set to the current time.

#### Scenario: Update existing cache entry after correction
- **WHEN** a cache entry exists for key "SBB MOBILE" and is updated with a new category
- **THEN** the existing entry SHALL be replaced with the new values and `updated_at` SHALL be current
