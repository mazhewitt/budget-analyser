use chrono::NaiveDate;
use serde::Deserialize;
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

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

#[derive(Debug, PartialEq)]
enum CsvFormat {
    Synthetic,
    AccountStatement,
    CreditCard,
}

fn detect_format(path: &Path) -> Result<CsvFormat, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).map_err(|e| format!("Failed to read first line: {}", e))?;

    // Strip UTF-8 BOM if present
    let line = first_line.strip_prefix("\u{feff}").unwrap_or(&first_line).trim();

    if line.starts_with("sep=;") {
        Ok(CsvFormat::CreditCard)
    } else if line.contains(":;") || line.starts_with("Account number:;") {
        Ok(CsvFormat::AccountStatement)
    } else {
        Ok(CsvFormat::Synthetic)
    }
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
    let format = detect_format(path)?;

    match format {
        CsvFormat::Synthetic => parse_synthetic(path),
        CsvFormat::AccountStatement => parse_account_statement(path),
        CsvFormat::CreditCard => parse_credit_card(path),
    }
}

fn parse_synthetic(path: &Path) -> Result<Vec<Transaction>, String> {
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

#[derive(Debug, Deserialize)]
struct AccountRecord {
    #[serde(rename = "Trade date")]
    trade_date: String,
    #[serde(rename = "Booking date")]
    booking_date: String,
    #[serde(rename = "Value date")]
    value_date: String,
    #[serde(rename = "Description1")]
    description1: String,
    #[serde(rename = "Description2")]
    description2: Option<String>,
    #[serde(rename = "Description3")]
    description3: Option<String>,
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename = "Debit")]
    debit: Option<String>,
    #[serde(rename = "Credit")]
    credit: Option<String>,
    #[serde(rename = "Transaction no.")]
    transaction_no: String,
}

fn parse_iso_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| format!("Failed to parse ISO date '{}': {}", s, e))
}

fn parse_account_statement(path: &Path) -> Result<Vec<Transaction>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = BufReader::new(file);
    let mut skip_lines = 0;
    let header_line;

    // Scan for header row
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).map_err(|e| format!("Failed to read line: {}", e))?;
        if bytes == 0 {
            return Err("Header row starting with 'Trade date;' not found".to_string());
        }
        if line.starts_with("Trade date;") {
            header_line = line;
            break;
        }
        skip_lines += 1;
    }

    // Re-open file and skip lines to use csv crate
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let lines = reader.lines().skip(skip_lines + 1); // Skip preamble AND the header row we already read
    let mut content = header_line;
    for line in lines {
        content.push_str(&line.map_err(|e| format!("Failed to read line: {}", e))?);
        content.push('\n');
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut transactions = Vec::new();

    for (i, result) in csv_reader.deserialize().enumerate() {
        let raw: AccountRecord = result
            .map_err(|e| format!("Failed to parse account row {}: {}", i + 1, e))?;

        let mut details_parts = Vec::new();
        if let Some(d2) = raw.description2 {
            let trimmed = d2.trim();
            if !trimmed.is_empty() { details_parts.push(trimmed.to_string()); }
        }
        if let Some(d3) = raw.description3 {
            let trimmed = d3.trim();
            if !trimmed.is_empty() { details_parts.push(trimmed.to_string()); }
        }

        let tx = Transaction {
            trade_date: parse_iso_date(&raw.trade_date)?,
            booking_date: parse_iso_date(&raw.booking_date)?,
            value_date: parse_iso_date(&raw.value_date)?,
            currency: raw.currency,
            debit: parse_amount(&raw.debit).map(|v| v.abs()),
            credit: parse_amount(&raw.credit).map(|v| v.abs()),
            balance: 0.0,
            transaction_id: raw.transaction_no,
            description: raw.description1,
            details: details_parts.join("; "),
            footnotes: String::new(),
        };

        transactions.push(tx);
    }

    Ok(transactions)
}

fn parse_credit_card(path: &Path) -> Result<Vec<Transaction>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    
    // Read the whole file to handle headers and skip 'sep=;'
    let mut lines = reader.lines();
    
    // Skip 'sep=;' line
    lines.next().ok_or("Empty file")?.map_err(|e| format!("Failed to read sep line: {}", e))?;
    
    // Read header line
    let header_line = lines.next().ok_or("Missing header line")?.map_err(|e| format!("Failed to read header line: {}", e))?;

    let mut content = header_line;
    content.push('\n');
    for line in lines {
        content.push_str(&line.map_err(|e| format!("Failed to read line: {}", e))?);
        content.push('\n');
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut transactions = Vec::new();

    for (i, result) in csv_reader.deserialize().enumerate() {
        let raw: CreditCardRecord = result
            .map_err(|e| format!("Failed to parse credit card row {}: {}", i + 1, e))?;

        // Filter out non-transaction rows
        if raw.purchase_date.trim().is_empty() || raw.booking_text.contains("DIRECT DEBIT") {
            continue;
        }

        let trade_date = parse_date(&raw.purchase_date)?;
        let booking_date = if raw.booked.trim().is_empty() {
            trade_date
        } else {
            parse_date(&raw.booked)?
        };

        let tx_id = generate_cc_transaction_id(&raw);

        let tx = Transaction {
            trade_date,
            booking_date,
            value_date: booking_date,
            currency: raw.currency,
            debit: parse_amount(&raw.debit).map(|v| v.abs()),
            credit: parse_amount(&raw.credit).map(|v| v.abs()),
            balance: 0.0,
            transaction_id: tx_id,
            description: raw.booking_text,
            details: raw.sector.unwrap_or_default(),
            footnotes: String::new(),
        };

        transactions.push(tx);
    }

    Ok(transactions)
}

#[derive(Debug, Deserialize)]
struct CreditCardRecord {
    #[serde(rename = "Purchase date")]
    purchase_date: String,
    #[serde(rename = "Booked")]
    booked: String,
    #[serde(rename = "Booking text")]
    booking_text: String,
    #[serde(rename = "Sector")]
    sector: Option<String>,
    #[serde(rename = "Currency")]
    currency: String,
    #[serde(rename = "Debit")]
    debit: Option<String>,
    #[serde(rename = "Credit")]
    credit: Option<String>,
}

fn generate_cc_transaction_id(raw: &CreditCardRecord) -> String {
    // FNV-1a 64-bit — stable across Rust versions (unlike DefaultHasher)
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for b in raw.purchase_date.as_bytes()
        .iter()
        .chain(b"|")
        .chain(raw.booking_text.as_bytes())
        .chain(b"|")
        .chain(raw.debit.as_deref().unwrap_or("").as_bytes())
        .chain(b"|")
        .chain(raw.credit.as_deref().unwrap_or("").as_bytes())
        .chain(b"|")
        .chain(raw.booked.as_bytes())
    {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("cc-{:016x}", hash)
}
