use rusqlite::{params, Connection, Result};
use std::path::Path;
use crate::categories::Category;
use crate::classifier::ClassificationResult;
use crate::csv_parser::Transaction;
use chrono::Utc;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct StoredTransaction {
    pub id: i64,
    pub date: String,
    pub raw_description: String,
    pub amount: f64,
    pub currency: String,
    pub merchant_name: String,
    pub category: String,
    pub source: String,
    pub confidence: f64,
    pub transaction_id: String,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // transactions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL,
                raw_description TEXT NOT NULL,
                amount REAL NOT NULL,
                currency TEXT NOT NULL,
                merchant_name TEXT NOT NULL,
                category TEXT NOT NULL,
                source TEXT NOT NULL,
                confidence REAL NOT NULL,
                transaction_id TEXT NOT NULL UNIQUE,
                import_batch TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // merchant_cache table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS merchant_cache (
                raw_key PRIMARY KEY,
                merchant_name TEXT NOT NULL,
                category TEXT NOT NULL,
                confidence REAL NOT NULL,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // import_log table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS import_log (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL,
                row_count INTEGER NOT NULL,
                imported_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Database { conn })
    }

    pub fn insert_transaction(&self, tx: &Transaction, classification: &ClassificationResult, import_batch: Option<&str>) -> Result<bool> {
        let amount = match (tx.debit, tx.credit) {
            (Some(d), _) => -d,
            (_, Some(c)) => c,
            _ => 0.0,
        };

        let created_at = Utc::now().to_rfc3339();
        let date = tx.trade_date.to_string();

        let res = self.conn.execute(
            "INSERT OR IGNORE INTO transactions (
                date, raw_description, amount, currency, merchant_name, 
                category, source, confidence, transaction_id, import_batch, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                date,
                tx.description,
                amount,
                tx.currency,
                classification.merchant,
                classification.category.to_string(),
                classification.source,
                classification.confidence,
                tx.transaction_id,
                import_batch,
                created_at
            ],
        );

        match res {
            Ok(rows) => Ok(rows > 0),
            Err(e) => Err(e),
        }
    }

    pub fn transaction_exists(&self, transaction_id: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT 1 FROM transactions WHERE transaction_id = ?")?;
        Ok(stmt.exists(params![transaction_id])?)
    }

    pub fn cache_lookup(&self, raw_key: &str) -> Result<Option<ClassificationResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT merchant_name, category, confidence, source FROM merchant_cache WHERE raw_key = ?"
        )?;

        let mut rows = stmt.query(params![raw_key])?;

        if let Some(row) = rows.next()? {
            let merchant: String = row.get(0)?;
            let category_str: String = row.get(1)?;
            let confidence: f64 = row.get(2)?;
            let source: String = row.get(3)?;

            let category: Category = serde_json::from_value(serde_json::Value::String(category_str))
                .unwrap_or(Category::Uncategorised);

            Ok(Some(ClassificationResult {
                merchant,
                category,
                confidence,
                source,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn cache_insert(&self, raw_key: &str, result: &ClassificationResult) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO merchant_cache (
                raw_key, merchant_name, category, confidence, source, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                raw_key,
                result.merchant,
                result.category.to_string(),
                result.confidence,
                result.source,
                now,
                now
            ],
        )?;
        Ok(())
    }

    pub fn log_import(&self, filename: &str, row_count: usize) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO import_log (filename, row_count, imported_at) VALUES (?, ?, ?)",
            params![filename, row_count as i64, now],
        )?;
        Ok(())
    }

    pub fn get_flagged_transactions(
        &self,
        threshold: f64,
        category: Option<&str>,
        since: Option<&str>,
        until: Option<&str>,
        merchant: Option<&str>,
    ) -> Result<Vec<StoredTransaction>> {
        let mut query = String::from(
            "SELECT id, date, raw_description, amount, currency, merchant_name, category, source, confidence, transaction_id 
             FROM transactions 
             WHERE (confidence < ? OR category = 'Other' OR category = 'Uncategorised')"
        );
        let mut params_vec: Vec<rusqlite::types::Value> = vec![rusqlite::types::Value::Real(threshold)];

        if let Some(cat) = category {
            query.push_str(" AND category = ?");
            params_vec.push(rusqlite::types::Value::Text(cat.to_string()));
        }
        if let Some(s) = since {
            query.push_str(" AND date >= ?");
            params_vec.push(rusqlite::types::Value::Text(s.to_string()));
        }
        if let Some(u) = until {
            query.push_str(" AND date <= ?");
            params_vec.push(rusqlite::types::Value::Text(u.to_string()));
        }
        if let Some(m) = merchant {
            query.push_str(" AND merchant_name LIKE ?");
            params_vec.push(rusqlite::types::Value::Text(format!("%{}%", m)));
        }

        query.push_str(" ORDER BY date ASC");

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(StoredTransaction {
                id: row.get(0)?,
                date: row.get(1)?,
                raw_description: row.get(2)?,
                amount: row.get(3)?,
                currency: row.get(4)?,
                merchant_name: row.get(5)?,
                category: row.get(6)?,
                source: row.get(7)?,
                confidence: row.get(8)?,
                transaction_id: row.get(9)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn update_transaction(
        &self,
        id: i64,
        merchant_name: &str,
        category: &str,
        confidence: f64,
        source: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE transactions 
             SET merchant_name = ?, category = ?, confidence = ?, source = ? 
             WHERE id = ?",
            params![merchant_name, category, confidence, source, id],
        )?;
        Ok(())
    }
}
