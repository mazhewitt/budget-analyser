use chrono::NaiveDate;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub trade_date: NaiveDate,
    pub booking_date: NaiveDate,
    pub value_date: NaiveDate,
    pub currency: String,
    pub debit: Option<f64>,
    pub credit: Option<f64>,
    pub balance: f64,
    pub transaction_id: String,
    pub description: String,
    pub details: String,
    pub footnotes: String,
}

#[derive(Debug, Deserialize)]
struct RawRecord {
    trade_date: String,
    booking_date: String,
    value_date: String,
    currency: String,
    debit: Option<String>,
    credit: Option<String>,
    balance: String,
    transaction_id: String,
    description: String,
    details: Option<String>,
    footnotes: Option<String>,
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%d.%m.%Y")
        .map_err(|e| format!("Failed to parse date '{}': {}", s, e))
}

fn parse_amount(s: &Option<String>) -> Option<f64> {
    s.as_ref().and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            trimmed.parse::<f64>().ok()
        }
    })
}

pub fn parse_csv(path: &Path) -> Result<Vec<Transaction>, String> {
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| format!("Failed to open CSV '{}': {}", path.display(), e))?;

    let mut transactions = Vec::new();

    for (i, result) in reader.deserialize().enumerate() {
        let raw: RawRecord = result
            .map_err(|e| format!("Failed to parse row {}: {}", i + 1, e))?;

        let tx = Transaction {
            trade_date: parse_date(&raw.trade_date)?,
            booking_date: parse_date(&raw.booking_date)?,
            value_date: parse_date(&raw.value_date)?,
            currency: raw.currency,
            debit: parse_amount(&raw.debit),
            credit: parse_amount(&raw.credit),
            balance: raw.balance.parse::<f64>()
                .map_err(|e| format!("Failed to parse balance on row {}: {}", i + 1, e))?,
            transaction_id: raw.transaction_id,
            description: raw.description,
            details: raw.details.unwrap_or_default(),
            footnotes: raw.footnotes.unwrap_or_default(),
        };

        transactions.push(tx);
    }

    Ok(transactions)
}
