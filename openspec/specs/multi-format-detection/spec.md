### Requirement: Auto-detect CSV format from file content

The system SHALL detect the CSV format by inspecting the first line of the file (after stripping any UTF-8 BOM). Detection rules:
1. If the first line is `sep=;` → credit card invoice format
2. If the first line contains `:;` (metadata key-value pattern) → account statement format
3. Otherwise → synthetic format (existing comma-separated parser)

The detected format SHALL determine which parsing path is used. No user flag or filename convention is required.

#### Scenario: Detect credit card invoice format

- **WHEN** a CSV file's first line is `sep=;`
- **THEN** the system SHALL use the credit card invoice parser

#### Scenario: Detect account statement format

- **WHEN** a CSV file's first line starts with `Account number:;` (possibly preceded by BOM)
- **THEN** the system SHALL use the account statement parser

#### Scenario: Detect synthetic format

- **WHEN** a CSV file's first line is `trade_date,booking_date,value_date,...`
- **THEN** the system SHALL use the existing comma-separated parser

#### Scenario: Handle UTF-8 BOM

- **WHEN** a CSV file begins with a UTF-8 BOM character (`\u{FEFF}`)
- **THEN** the system SHALL strip the BOM before format detection
