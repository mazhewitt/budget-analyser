2## Context

The CSV parser (`src/csv_parser.rs`) handles a single format: the synthetic UBS account statement (comma-separated, flat headers, DD.MM.YYYY dates). Real UBS exports come in two different formats with semicolons, header preambles, and different column layouts. The `Transaction` struct and all downstream code (classifier, cache, DB) are format-agnostic — only the parsing layer needs to change.

## Goals / Non-Goals

**Goals:**
- Import real UBS account statements and credit card invoices alongside synthetic data
- Auto-detect format without user intervention
- Produce identical `Transaction` structs regardless of source format
- Generate stable, deterministic transaction IDs for credit card rows (which lack them)
- Maintain backward compatibility with existing synthetic format

**Non-Goals:**
- Supporting non-UBS bank formats
- Changing the `Transaction` struct shape or downstream pipeline
- Parsing multi-currency exchange rate details from credit card invoices
- Tracking which card holder made a purchase (credit card `Account/Cardholder` field)

## Decisions

### 1. Format detection by header sniffing

**Decision**: Read the first line of the file (after stripping BOM) and dispatch:
- Starts with `sep=;` → credit card invoice parser
- Starts with `Account number:;` or contains `:;` metadata pattern → account statement parser
- Otherwise → existing synthetic parser (comma-separated)

**Rationale**: Simple, deterministic, no user flag needed. The three formats have unambiguous first lines. Avoids adding CLI arguments or filename conventions.

### 2. Account statement parsing strategy

**Decision**: Read the file as raw text, skip lines until we find the header row (line starting with `Trade date;`), then feed the remainder to `csv::ReaderBuilder` with semicolon delimiter and flexible mode enabled.

**Rationale**: The 8-line metadata preamble isn't fixed — safer to scan for the header row than hardcode a skip count. The header row has a trailing semicolon, so flexible mode handles the extra empty column.

**Field mapping**:
- `trade_date` ← `Trade date` (parse as `yyyy-mm-dd`)
- `booking_date` ← `Booking date`
- `value_date` ← `Value date`
- `currency` ← `Currency`
- `debit` / `credit` ← `Debit` / `Credit` (parse as optional f64, strip minus signs)
- `transaction_id` ← `Transaction no.`
- `description` ← `Description1` (primary merchant/payee info)
- `details` ← `Description2` + `Description3` concatenated (payment method + reference info, useful for classification)
- `balance` ← 0.0 (unused downstream)

### 3. Credit card invoice parsing strategy

**Decision**: Skip `sep=;` line, parse with semicolon delimiter. Filter out rows where `Purchase date` is empty (balance carried forward, summary rows) and rows with booking text "DIRECT DEBIT".

**Field mapping**:
- `trade_date` ← `Purchase date` (parse as `dd.mm.yyyy`)
- `booking_date` / `value_date` ← `Booked` date (or `Purchase date` if missing)
- `currency` ← `Currency`
- `debit` ← `Debit` (when present, it's a spend)
- `credit` ← `Credit` (when present, it's a refund/payment)
- `description` ← `Booking text`
- `details` ← `Sector` (e.g. "Grocery stores", "Restaurants" — valuable classification signal)
- `transaction_id` ← generated (see below)
- `balance` ← 0.0

### 4. Transaction ID generation for credit card

**Decision**: Generate a deterministic ID by hashing `purchase_date|booking_text|amount|booked_date` using a simple hash (e.g., first 16 hex chars of a hash, prefixed with `cc-`).

**Rationale**: Credit card exports have no transaction ID. The hash must be stable across re-imports for dedup to work. The combination of date + merchant + amount + booking date is unique enough for typical statements. Prefix `cc-` distinguishes generated IDs from real account statement IDs.

**Alternative considered**: Sequential counter — rejected because it breaks on re-import.

### 5. Test fixture data

**Decision**: Create minimal but realistic test fixtures in `data/test/` that mirror the real file structures (headers, preambles, semicolons, date formats) with anonymised data. One file per format.

**Rationale**: Regression tests need stable fixtures. Using real data in tests risks privacy leakage. The fixtures should exercise edge cases (empty optional fields, multi-description concatenation, filtered rows in credit card).

## Risks / Trade-offs

- **[Semicolons inside quoted fields]** → UBS exports use semicolons both as delimiters and within quoted Description fields (e.g. `"addr1;addr2"`). The `csv` crate handles RFC 4180 quoting correctly, so this should work. Verify with real data.
- **[Hash collisions for credit card IDs]** → Two purchases at the same merchant for the same amount on the same day would collide. Mitigation: include the `Booked` date which may differ. Extremely rare in practice. If needed, add row index as tiebreaker.
- **[BOM handling]** → The real `transactions.csv` starts with a UTF-8 BOM (`\u{FEFF}`). Must strip before sniffing. The `csv` crate doesn't strip BOM automatically when reading from a cursor.
