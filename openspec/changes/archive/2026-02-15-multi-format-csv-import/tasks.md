## 1. Test Fixtures

- [x] 1.1 Create `data/test/account-statement.csv` — realistic anonymised UBS account statement with metadata preamble, semicolon delimiter, ISO dates, multi-description fields, quoted fields with embedded semicolons (5-8 rows)
- [x] 1.2 Create `data/test/credit-card-invoice.csv` — realistic anonymised UBS credit card invoice with `sep=;` header, balance-brought-forward row, DIRECT DEBIT row, sector field, DD.MM.YYYY dates, debit and credit rows (5-8 rows)

## 2. Format Detection

- [x] 2.1 Add `detect_format()` function in `csv_parser.rs` that reads the first line (stripping UTF-8 BOM), returns an enum `CsvFormat { Synthetic, AccountStatement, CreditCard }`
- [x] 2.2 Update `parse_csv()` to call `detect_format()` and dispatch to the appropriate parser function

## 3. Account Statement Parser

- [x] 3.1 Implement `parse_account_statement()` — scan for header row starting with `Trade date;`, skip preamble, use semicolon delimiter with flexible mode
- [x] 3.2 Map fields: Trade date (YYYY-MM-DD) → trade_date, Description1 → description, Description2+Description3 → details, Transaction no. → transaction_id
- [x] 3.3 Handle quoted fields with embedded semicolons and trailing semicolons in header

## 4. Credit Card Invoice Parser

- [x] 4.1 Implement `parse_credit_card()` — skip `sep=;` line, semicolon delimiter, filter out rows with empty Purchase date and "DIRECT DEBIT" booking text
- [x] 4.2 Map fields: Purchase date (DD.MM.YYYY) → trade_date, Booking text → description, Sector → details, Booked → booking_date/value_date
- [x] 4.3 Implement `generate_cc_transaction_id()` — hash purchase_date+booking_text+amount+booked_date, prefix with `cc-`

## 5. Verification

- [x] 5.1 `cargo build` — ensure compilation succeeds
- [x] 5.2 Test import of `data/test/account-statement.csv` and verify correct parsing
- [x] 5.3 Test import of `data/test/credit-card-invoice.csv` and verify correct parsing, dedup on re-import
- [x] 5.4 Regression test: import `data/synthetic-ubstransactions-feb2026.csv` still works
- [x] 5.5 Test import of real data from `data/real_data/` — both files
