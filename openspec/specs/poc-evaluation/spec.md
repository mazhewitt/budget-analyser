## ADDED Requirements

### Requirement: Load ground-truth labels from TOML file

The system SHALL load manually-labelled ground-truth data from `data/ground-truth.toml`. Each entry SHALL map a transaction_id to an expected category and expected merchant name.

#### Scenario: Load ground-truth file

- **WHEN** the evaluator is initialised with path `data/ground-truth.toml`
- **THEN** it SHALL parse all entries into a lookup table keyed by transaction_id

### Requirement: Measure classification accuracy

The system SHALL compare LLM classification results against ground-truth labels and compute overall accuracy as the percentage of transactions where the assigned category matches the expected category. Accuracy SHALL be computed separately for auto-accepted transactions (those above the confidence threshold) and for all transactions.

#### Scenario: Compute accuracy above threshold

- **WHEN** 20 of 24 transactions are above the confidence threshold and 18 of those match ground truth
- **THEN** the auto-accepted accuracy SHALL be reported as 90%

#### Scenario: Compute overall accuracy

- **WHEN** 20 of 24 total transactions match ground truth
- **THEN** the overall accuracy SHALL be reported as 83.3%

### Requirement: Measure flagging rate

The system SHALL compute the flagging rate as the percentage of transactions with confidence below a configurable threshold (default 0.80). These are transactions that would require manual review.

#### Scenario: Compute flagging rate

- **WHEN** 5 of 24 transactions have confidence below the threshold
- **THEN** the flagging rate SHALL be reported as 20.8%

### Requirement: Detect false-confidence misclassifications

The system SHALL identify transactions where the LLM reported high confidence (above threshold) but the assigned category does not match the ground-truth label. These false-confidence results SHALL be reported individually with full details.

#### Scenario: Flag false-confidence result

- **WHEN** a transaction has confidence 0.95 but the category does not match ground truth
- **THEN** it SHALL appear in the false-confidence report with transaction details, predicted category, expected category, and confidence score

### Requirement: Print evaluation summary report

The system SHALL print a summary report to stdout containing: overall accuracy, auto-accepted accuracy, flagging rate, number of false-confidence results, and a per-category breakdown. The report SHALL also list all misclassified transactions for manual inspection.

#### Scenario: Summary report output

- **WHEN** evaluation completes
- **THEN** the report SHALL display all metrics and list each misclassified transaction with its predicted vs expected category
