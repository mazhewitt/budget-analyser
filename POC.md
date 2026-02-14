# Proof of Concept Plan

## What We're Testing

The POC targets the two genuinely uncertain parts of this project:

1. **Can a local 8B-class LLM reliably classify real UBS merchant strings?** The cryptic, truncated, Swiss-German merchant descriptions in UBS exports are the core challenge. Everything else (CSV parsing, SQLite, caching) is known-quantity engineering.

2. **Does the category schema hold up against real data?** A schema that looks clean on paper often falls apart when confronted with ambiguous real-world transactions. The POC will surface edge cases and missing categories early.

## What We're Not Testing

CSV parsing, database design, and batch orchestration are straightforward and don't need a POC. We'll build those properly in the main implementation.

## Approach

### Step 1: Prepare Test Data

Use the included synthetic dataset `data/synthetic-ubstransactions-feb2026.csv` as the POC test export. The CSV mirrors a UBS monthly export and contains a mix of realistic transactions (SBB/SBB Mobile/TWINT, e-bills with QRR, webshop purchases, standing orders, transfers, incoming credits and a large card posting). Treat this file as the single-month CSV for all POC runs.

Why use this file:
- It reproduces the cryptic/multi-field merchant strings we need to normalise and categorise.
- It includes EBILL/QRR lines, TWINT/SBB mobile small payments, refunds/credits and a large card/statement posting — good coverage for edge cases.

Preview (first 10 rows of `data/synthetic-ubstransactions-feb2026.csv`):

```csv
trade_date,booking_date,value_date,currency,debit,credit,balance,transaction_id,description
12.02.2026,12.02.2026,12.02.2026,CHF,-877.45,,1483.82,22932043DJ7410461,Steuerverwaltung EBILL-RECHT
12.02.2026,12.02.2026,12.02.2026,CHF,-535.70,,2361.27,21920435DJ7410458,Steuerverwaltung EBILL-RECHT
11.02.2026,11.02.2026,11.02.2026,CHF,-11.80,,2896.97,999342402GK607d7402,SBB MOBILE
09.02.2026,09.02.2026,09.02.2026,CHF,-11.80,,2908.77,9992040GK484446481,SBB MOBILE
09.02.2026,09.02.2026,09.02.2026,CHF,-150.00,,2920.57,99925537LK3754207,Michy Holder (transfer)
06.02.2026,06.02.2026,06.02.2026,CHF,-11.80,,4420.57,9992037G454K2742652,SBB MOBILE
05.02.2026,05.02.2026,05.02.2026,CHF,-11.80,,4432.37,9992036GK1943846,SBB MOBILE
04.02.2026,04.02.2026,04.02.2026,CHF,-15.70,,4444.17,249203455DJ6603690,Vivao Sympa EBILL-RECHT
03.02.2026,03.02.2026,03.02.2026,CHF,-15.60,,4459.87,9992035GK126543731,SBB MOBILE
03.02.2026,03.02.2026,03.02.2026,CHF,-362.15,,4475.47,9992034334GK0875121,DIGITEC GALAXUS
```

Labeling guidance:
- Manually label these 24 transactions as the POC ground-truth (start here, expand if you need 50–100 examples).
- Ensure at least one labelled example per category (groceries, transport, utilities, subscriptions, transfers, card payments, refunds).

How to run the POC against this file:
- Point the test runner at `data/synthetic-ubstransactions-feb2026.csv` (or import it into the test DB) and run the classification batch.
- Use the usual review workflow to inspect low-confidence / flagged items.

(Everything else in the POC — prompts, cache tests and evaluation steps — remains unchanged.)


### Step 2: Test LLM Classification

Run each test transaction through the local LLM with the system prompt containing the category schema. For each transaction, capture:

- The merchant name the LLM extracts
- The category it assigns
- The confidence score it reports
- Whether it matches the manual ground truth label

Test with at least two models (e.g. Llama 3.1 8B and Qwen 2.5 7B) to see if one handles Swiss merchant strings notably better than the other.

### Step 3: Evaluate Prompt Design

The system prompt is the main lever. Iterate on:

- How the category schema is presented (descriptions, examples, or both)
- Whether including amount context measurably improves accuracy
- Whether few-shot examples of real UBS merchant strings help with the cryptic formatting
- The confidence calibration — does the model's reported confidence actually correlate with accuracy?

### Step 4: Test the Cache Key Strategy

The merchant cache only works if the normalisation strategy is right. UBS merchant strings for the same merchant often vary by trailing reference numbers, card fragments, or location codes. Test whether the normalisation approach (strip trailing digits, uppercase, etc.) correctly groups variants of the same merchant without false collisions.

For example, these should all resolve to the same cache entry:
- `MIGROS BASEL M 01234 KARTE 1234`
- `MIGROS BASEL M 56789 KARTE 1234`
- `MIGROS BASEL M 01234 KARTE 5678`

But these should not:
- `COOP PRONTO BHFSTR ZURICH` (convenience store / café)
- `COOP CITY BASEL` (department store)

### Step 5: Measure the Confidence Threshold

Run the full test set and plot accuracy against the model's self-reported confidence. The goal is to find the threshold where auto-accepted classifications are reliable enough that manual review is only needed for genuinely ambiguous cases — probably somewhere between 0.75 and 0.90.

## Success Criteria

The POC is successful if:

- **Accuracy ≥ 85%** on the labelled test set for auto-accepted transactions (those above the confidence threshold)
- **Flagging rate ≤ 20%** — no more than 1 in 5 transactions should need manual review
- **Zero miscategorised high-confidence results** — if the model says it's confident, it should be right. False confidence is worse than low confidence.
- **Cache key collisions are rare** — the normalisation strategy groups correctly without merging distinct merchants

## Expected Outcome

Most likely: the LLM handles common Swiss merchants well (SBB, Migros, Coop, Swisscom, etc.) but struggles with obscure local businesses, German-language medical descriptions, and heavily truncated strings. The confidence threshold will need tuning. The category schema will need at least one or two adjustments once real data reveals gaps.

If accuracy is below 70%, we either need a larger model, a richer prompt with more Swiss-specific few-shot examples, or a hybrid approach where the LLM only extracts/normalises the merchant name and category assignment is rule-based.

## Next Steps After POC

Once the POC validates the approach, hand off to Claude Code for implementation:

- Full CSV parser handling UBS export format variations
- SQLite schema and import pipeline with duplicate detection
- Batch processing with cache integration
- Interactive review mode for flagged transactions
- Basic query interface for spending analysis
