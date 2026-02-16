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
    pub _source: String,
    pub confidence: f64,
    pub _transaction_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CategoryInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FewShotExample {
    pub _merchant_pattern: String,
    pub raw_description: String,
    pub correct_merchant: String,
    pub correct_category: String,
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

        // few_shot_examples table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS few_shot_examples (
                id INTEGER PRIMARY KEY,
                merchant_pattern TEXT NOT NULL UNIQUE,
                raw_description TEXT NOT NULL,
                correct_merchant TEXT NOT NULL,
                correct_category TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // categories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                name TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Seed categories if empty
        {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM categories")?;
            let count: i64 = stmt.query_row([], |row| row.get(0))?;
            if count == 0 {
                let now = Utc::now().to_rfc3339();
                for cat in Category::all() {
                    conn.execute(
                        "INSERT INTO categories (name, description, created_at) VALUES (?, ?, ?)",
                        params![cat.to_string(), cat.description(), now],
                    )?;
                }
            }
        }

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
                classification.category,
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
            let category: String = row.get(1)?;
            let confidence: f64 = row.get(2)?;
            let source: String = row.get(3)?;

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
                result.category,
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
                _source: row.get(7)?,
                confidence: row.get(8)?,
                _transaction_id: row.get(9)?,
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

    pub fn insert_few_shot_example(
        &self,
        merchant_pattern: &str,
        raw_description: &str,
        correct_merchant: &str,
        correct_category: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO few_shot_examples (
                merchant_pattern, raw_description, correct_merchant, correct_category, created_at
            ) VALUES (?, ?, ?, ?, ?)",
            params![merchant_pattern, raw_description, correct_merchant, correct_category, now],
        )?;
        Ok(())
    }

    pub fn get_few_shot_examples(&self) -> Result<Vec<FewShotExample>> {
        let mut stmt = self.conn.prepare(
            "SELECT merchant_pattern, raw_description, correct_merchant, correct_category FROM few_shot_examples"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FewShotExample {
                _merchant_pattern: row.get(0)?,
                raw_description: row.get(1)?,
                correct_merchant: row.get(2)?,
                correct_category: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn delete_llm_cache_entry(&self, raw_key: &str) -> Result<bool> {
        let rows = self.conn.execute(
            "DELETE FROM merchant_cache WHERE raw_key = ? AND source = 'llm'",
            params![raw_key],
        )?;
        Ok(rows > 0)
    }

    pub fn list_categories(&self) -> Result<Vec<CategoryInfo>> {
        let mut stmt = self.conn.prepare("SELECT name, description FROM categories ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(CategoryInfo {
                name: row.get(0)?,
                description: row.get(1)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn add_category(&self, name: &str, description: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO categories (name, description, created_at) VALUES (?, ?, ?)",
            params![name, description, now],
        )?;
        Ok(())
    }
}
