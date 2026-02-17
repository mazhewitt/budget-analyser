pub mod import;

use sqlx::{Row, SqlitePool};

pub use import::{CategoryInfo, Database, FewShotExample, StoredTransaction};

#[derive(Debug, Clone)]
pub struct CategoryCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone)]
pub struct DataSummary {
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub total_transactions: i64,
    pub categories: Vec<CategoryCount>,
}

pub async fn connect_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePool::connect(database_url).await
}

pub async fn load_data_summary(pool: &SqlitePool) -> Result<DataSummary, sqlx::Error> {
    let row = sqlx::query(
        "SELECT MIN(date) as min_date, MAX(date) as max_date, COUNT(*) as total FROM transactions",
    )
    .fetch_one(pool)
    .await?;

    let min_date: Option<String> = row.get("min_date");
    let max_date: Option<String> = row.get("max_date");
    let total: i64 = row.get("total");

    let cat_rows = sqlx::query(
        "SELECT category, COUNT(*) as count FROM transactions GROUP BY category ORDER BY count DESC",
    )
    .fetch_all(pool)
    .await?;

    let categories = cat_rows
        .into_iter()
        .map(|r| CategoryCount {
            name: r.get::<Option<String>, _>("category").unwrap_or_else(|| "Unknown".to_string()),
            count: r.get::<i64, _>("count"),
        })
        .collect();

    Ok(DataSummary {
        min_date,
        max_date,
        total_transactions: total,
        categories,
    })
}
