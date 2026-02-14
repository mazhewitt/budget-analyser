## 1. Project Setup

- [x] 1.1 Add dependencies to Cargo.toml: csv, serde, serde_json, reqwest (with blocking or tokio), toml, chrono
- [x] 1.2 Create module structure: csv_parser.rs, categories.rs, classifier.rs, cache.rs, evaluator.rs

## 2. Category Schema

- [x] 2.1 Define Category enum with variants: Groceries, Transport, Utilities, Subscriptions, Transfers, CardPayments, Insurance, Taxes, Shopping, Dining, Income, Uncategorised
- [x] 2.2 Implement Display, Serialize/Deserialize for Category enum
- [x] 2.3 Add category descriptions and prompt-ready schema serialisation method

## 3. CSV Parsing

- [x] 3.1 Define Transaction struct with all 11 UBS CSV fields (dates as NaiveDate, amounts as Option<f64>)
- [x] 3.2 Implement CSV parser that reads UBS format with DD.MM.YYYY date parsing
- [x] 3.3 Verify parser returns 24 records from synthetic-ubstransactions-feb2026.csv

## 4. Merchant Normalisation and Cache

- [x] 4.1 Implement normalise_merchant_key function: uppercase, strip EBILL-RECHT/EBILL/KARTE suffixes, remove trailing digit-only tokens, collapse whitespace
- [x] 4.2 Write tests for normalisation: SBB MOBILE grouping, EBILL stripping, COOP variants stay separate, MIGROS reference number stripping
- [x] 4.3 Implement in-memory HashMap cache with lookup and insert

## 5. LLM Classification

- [x] 5.1 Define ClassificationResult struct (merchant: String, category: Category, confidence: f64)
- [x] 5.2 Build system prompt with category schema and few-shot UBS merchant examples
- [x] 5.3 Implement Ollama API client: POST /api/chat with JSON mode, configurable model and endpoint
- [x] 5.4 Parse JSON response into ClassificationResult, fallback to Uncategorised with confidence 0.0 on parse failure

## 6. Ground Truth and Evaluation

- [x] 6.1 Create data/ground-truth.toml with manual labels for all 24 synthetic transactions
- [x] 6.2 Implement ground-truth TOML loader
- [x] 6.3 Implement accuracy computation (overall and auto-accepted above confidence threshold)
- [x] 6.4 Implement flagging rate computation
- [x] 6.5 Implement false-confidence detection (high confidence + wrong category)
- [x] 6.6 Implement summary report printer: metrics, per-category breakdown, misclassified transaction list

## 7. CLI Orchestration

- [x] 7.1 Wire up main.rs: parse CSV → classify each transaction (with cache) → evaluate → print report
- [x] 7.2 Add CLI arguments: CSV path, model name, Ollama endpoint URL, confidence threshold
- [x] 7.3 End-to-end test: run full POC pipeline against synthetic dataset with a running Ollama instance
