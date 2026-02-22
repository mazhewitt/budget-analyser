use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use sqlx::sqlite::SqliteArguments;
use sqlx::Arguments;

use crate::ai::llm::ToolDefinition;

#[derive(Debug, Clone, Serialize)]
pub struct ToolOutput {
	pub summary: String,
	pub charts: Vec<ChartSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChartSpec {
	#[serde(rename = "type")]
	pub chart_type: String,
	pub title: String,
	pub data: ChartData,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChartData {
	pub labels: Vec<String>,
	pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dataset {
	pub name: String,
	pub values: Vec<f64>,
}

#[derive(Debug)]
pub enum ToolError {
	InvalidInput(String),
	Query(sqlx::Error),
}

impl From<sqlx::Error> for ToolError {
	fn from(err: sqlx::Error) -> Self {
		ToolError::Query(err)
	}
}

pub struct ToolRegistry {
	definitions: Vec<ToolDefinition>,
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            definitions: self.definitions.clone(),
        }
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            definitions: vec![
				ToolDefinition {
					name: "spending_by_category".to_string(),
					description: "Summarise spending by category with optional year/month filters.".to_string(),
					input_schema: json!({
						"type": "object",
						"properties": {
							"year": { "type": "integer" },
							"month": { "type": "integer" }
						},
						"additionalProperties": false
					}),
				},
				ToolDefinition {
					name: "monthly_trend".to_string(),
					description: "Summarise monthly spending trend with optional category/year filters.".to_string(),
					input_schema: json!({
						"type": "object",
						"properties": {
							"category": { "type": "string" },
							"year": { "type": "integer" }
						},
						"additionalProperties": false
					}),
				},
				ToolDefinition {
					name: "merchant_breakdown".to_string(),
					description: "Show top merchants within a category with optional top_n.".to_string(),
					input_schema: json!({
						"type": "object",
						"properties": {
							"category": { "type": "string" },
							"top_n": { "type": "integer" }
						},
						"required": ["category"],
						"additionalProperties": false
					}),
				},
				ToolDefinition {
					name: "search_transactions".to_string(),
					description: "Search transactions by merchant name or description. Returns total spend, count, average, date range, merchant name variants, and a monthly spending chart.".to_string(),
					input_schema: json!({
						"type": "object",
						"properties": {
							"search": { "type": "string", "description": "Search term to match against merchant name or raw description" },
							"category": { "type": "string" },
							"year": { "type": "integer" },
							"month": { "type": "integer" }
						},
						"required": ["search"],
						"additionalProperties": false
					}),
				},
				ToolDefinition {
					name: "list_transactions".to_string(),
					description: "List individual transactions matching a search term. Returns date, amount, merchant, and raw description for each transaction.".to_string(),
					input_schema: json!({
						"type": "object",
						"properties": {
							"search": { "type": "string", "description": "Search term to match against merchant name or raw description" },
							"category": { "type": "string" },
							"year": { "type": "integer" },
							"month": { "type": "integer" },
							"limit": { "type": "integer", "description": "Max rows to return (default 50)" }
						},
						"required": ["search"],
						"additionalProperties": false
					}),
				},
				ToolDefinition {
					name: "income_vs_spending".to_string(),
					description: "Compare monthly income vs spending with optional year filter.".to_string(),
					input_schema: json!({
						"type": "object",
						"properties": {
							"year": { "type": "integer" }
						},
						"additionalProperties": false
					}),
				},
			],
		}
	}

	pub fn definitions(&self) -> &[ToolDefinition] {
		&self.definitions
	}

	pub async fn run(
		&self,
		pool: &SqlitePool,
		name: &str,
		input: serde_json::Value,
	) -> Result<ToolOutput, ToolError> {
		match name {
			"spending_by_category" => spending_by_category(pool, input).await,
			"monthly_trend" => monthly_trend(pool, input).await,
			"merchant_breakdown" => merchant_breakdown(pool, input).await,
			"income_vs_spending" => income_vs_spending(pool, input).await,
			"search_transactions" => search_transactions(pool, input).await,
			"list_transactions" => list_transactions(pool, input).await,
			_ => Err(ToolError::InvalidInput(format!("Unknown tool: {}", name))),
		}
	}
}

#[derive(Debug, Deserialize)]
struct SpendingByCategoryInput {
	year: Option<i32>,
	month: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MonthlyTrendInput {
	category: Option<String>,
	year: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct MerchantBreakdownInput {
	category: String,
	top_n: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct IncomeVsSpendingInput {
	year: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct TransactionSearchInput {
	search: String,
	category: Option<String>,
	year: Option<i32>,
	month: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ListTransactionsInput {
	search: String,
	category: Option<String>,
	year: Option<i32>,
	month: Option<u32>,
	limit: Option<i64>,
}

/// Build WHERE clause and params for transaction search.
/// Matches `search` term against merchant_name and raw_description (case-insensitive LIKE).
fn build_search_conditions(search: &str, category: &Option<String>, year: &Option<i32>, month: &Option<u32>) -> (Vec<String>, Vec<String>) {
	let mut conditions = vec![
		"amount < 0".to_string(),
		"(LOWER(merchant_name) LIKE '%' || LOWER(?) || '%' OR LOWER(raw_description) LIKE '%' || LOWER(?) || '%')".to_string(),
	];
	let mut params = vec![search.to_string(), search.to_string()];

	if let Some(cat) = category {
		conditions.push("category = ?".to_string());
		params.push(cat.clone());
	}
	if let Some(y) = year {
		conditions.push("strftime('%Y', date) = ?".to_string());
		params.push(y.to_string());
	}
	if let Some(m) = month {
		conditions.push("strftime('%m', date) = ?".to_string());
		params.push(format!("{:02}", m));
	}

	(conditions, params)
}

async fn spending_by_category(
	pool: &SqlitePool,
	input: serde_json::Value,
) -> Result<ToolOutput, ToolError> {
	let input: SpendingByCategoryInput = serde_json::from_value(input)
		.map_err(|e| ToolError::InvalidInput(e.to_string()))?;

	let mut conditions = vec!["amount < 0".to_string(), "category != 'Transfers'".to_string()];
	let mut params: Vec<String> = Vec::new();

	if let Some(year) = input.year {
		conditions.push("strftime('%Y', date) = ?".to_string());
		params.push(year.to_string());
	}
	if let Some(month) = input.month {
		let month_str = format!("{:02}", month);
		conditions.push("strftime('%m', date) = ?".to_string());
		params.push(month_str);
	}

	let where_clause = if conditions.is_empty() {
		"".to_string()
	} else {
		format!("WHERE {}", conditions.join(" AND "))
	};

	let query = format!(
		"SELECT category, -SUM(amount) as spend, COUNT(*) as count\n         FROM transactions\n         {}\n         GROUP BY category\n         ORDER BY spend DESC",
		where_clause
	);

	let mut args = SqliteArguments::default();
	for param in params {
		let _ = args.add(param);
	}
	let rows = sqlx::query_with(&query, args).fetch_all(pool).await?;

	let mut labels = Vec::new();
	let mut values = Vec::new();
	let mut summary_parts = Vec::new();

	for row in rows.iter() {
		let category: String = row.try_get("category").unwrap_or_else(|_| "Unknown".to_string());
		let spend: f64 = row.try_get("spend").unwrap_or(0.0);
		let count: i64 = row.try_get("count").unwrap_or(0);
		labels.push(category.clone());
		values.push(spend);
		summary_parts.push(format!("{} CHF {:.2} ({} tx)", category, spend, count));
	}

	let summary = if summary_parts.is_empty() {
		"No spending data found for the requested period.".to_string()
	} else {
		format!("Spending by category: {}.", summary_parts.join(", "))
	};

	let chart = ChartSpec {
		chart_type: "bar_h".to_string(),
		title: "Spending by Category".to_string(),
		data: ChartData {
			labels,
			datasets: vec![Dataset {
				name: "CHF".to_string(),
				values,
			}],
		},
		height: Some(320),
	};

	Ok(ToolOutput {
		summary,
		charts: vec![chart],
	})
}

async fn monthly_trend(
	pool: &SqlitePool,
	input: serde_json::Value,
) -> Result<ToolOutput, ToolError> {
	let input: MonthlyTrendInput = serde_json::from_value(input)
		.map_err(|e| ToolError::InvalidInput(e.to_string()))?;

	let mut conditions = vec!["amount < 0".to_string(), "category != 'Transfers'".to_string()];
	let mut params: Vec<String> = Vec::new();

	if let Some(category) = input.category {
		conditions.push("category = ?".to_string());
		params.push(category);
	}
	if let Some(year) = input.year {
		conditions.push("strftime('%Y', date) = ?".to_string());
		params.push(year.to_string());
	}

	let where_clause = if conditions.is_empty() {
		"".to_string()
	} else {
		format!("WHERE {}", conditions.join(" AND "))
	};

	let query = format!(
		"SELECT strftime('%Y-%m', date) as month, -SUM(amount) as spend\n         FROM transactions\n         {}\n         GROUP BY month\n         ORDER BY month ASC",
		where_clause
	);

	let mut args = SqliteArguments::default();
	for param in params {
		let _ = args.add(param);
	}
	let rows = sqlx::query_with(&query, args).fetch_all(pool).await?;

	let mut labels = Vec::new();
	let mut values = Vec::new();

	for row in rows.iter() {
		let month: String = row.try_get("month").unwrap_or_else(|_| "Unknown".to_string());
		let spend: f64 = row.try_get("spend").unwrap_or(0.0);
		labels.push(month);
		values.push(spend);
	}

	let summary = if labels.is_empty() {
		"No monthly trend data found for the requested period.".to_string()
	} else {
		format!("Monthly spending recorded across {} months.", labels.len())
	};

	let chart = ChartSpec {
		chart_type: "bar".to_string(),
		title: "Monthly Spending Trend".to_string(),
		data: ChartData {
			labels,
			datasets: vec![Dataset {
				name: "CHF".to_string(),
				values,
			}],
		},
		height: Some(320),
	};

	Ok(ToolOutput {
		summary,
		charts: vec![chart],
	})
}

async fn merchant_breakdown(
	pool: &SqlitePool,
	input: serde_json::Value,
) -> Result<ToolOutput, ToolError> {
	let input: MerchantBreakdownInput = serde_json::from_value(input)
		.map_err(|e| ToolError::InvalidInput(e.to_string()))?;
	let top_n = input.top_n.unwrap_or(15).max(1) as usize;

	let query = r#"
		SELECT merchant_name as merchant, -SUM(amount) as spend, COUNT(*) as count, AVG(-amount) as avg_spend
		FROM transactions
		WHERE amount < 0 AND category = ?
		GROUP BY merchant_name
		ORDER BY spend DESC
	"#;

	let rows = sqlx::query(query)
		.bind(&input.category)
		.fetch_all(pool)
		.await?;

	let mut entries: Vec<(String, f64, i64, f64)> = rows
		.iter()
		.map(|row| {
			let merchant: String = row.try_get("merchant").unwrap_or_else(|_| "Unknown".to_string());
			let spend: f64 = row.try_get("spend").unwrap_or(0.0);
			let count: i64 = row.try_get("count").unwrap_or(0);
			let avg_spend: f64 = row.try_get("avg_spend").unwrap_or(0.0);
			(merchant, spend, count, avg_spend)
		})
		.collect();

	if entries.is_empty() {
		return Ok(ToolOutput {
			summary: "No merchant data found for that category.".to_string(),
			charts: Vec::new(),
		});
	}

	let mut top_entries = Vec::new();
	let mut other_spend = 0.0;
	let mut other_count = 0;

	for (idx, entry) in entries.drain(..).enumerate() {
		if idx < top_n {
			top_entries.push(entry);
		} else {
			other_spend += entry.1;
			other_count += entry.2;
		}
	}

	if other_count > 0 {
		top_entries.push(("Other".to_string(), other_spend, other_count, 0.0));
	}

	let mut labels = Vec::new();
	let mut values = Vec::new();
	let mut summary_parts = Vec::new();

	for (merchant, spend, count, avg) in &top_entries {
		labels.push(merchant.clone());
		values.push(*spend);
		if *merchant != "Other" {
			summary_parts.push(format!("{} CHF {:.2} ({} tx, avg CHF {:.2})", merchant, spend, count, avg));
		}
	}

	let summary = format!(
		"Top merchants for {}: {}.",
		input.category,
		summary_parts.join(", ")
	);

	let bar_chart = ChartSpec {
		chart_type: "bar_h".to_string(),
		title: format!("{} by Merchant", input.category),
		data: ChartData {
			labels: labels.clone(),
			datasets: vec![Dataset {
				name: "CHF".to_string(),
				values: values.clone(),
			}],
		},
		height: Some(340),
	};

	let pie_chart = ChartSpec {
		chart_type: "pie".to_string(),
		title: format!("{} Share", input.category),
		data: ChartData {
			labels,
			datasets: vec![Dataset {
				name: "CHF".to_string(),
				values,
			}],
		},
		height: Some(300),
	};

	Ok(ToolOutput {
		summary,
		charts: vec![bar_chart, pie_chart],
	})
}

async fn income_vs_spending(
	pool: &SqlitePool,
	input: serde_json::Value,
) -> Result<ToolOutput, ToolError> {
	let input: IncomeVsSpendingInput = serde_json::from_value(input)
		.map_err(|e| ToolError::InvalidInput(e.to_string()))?;

	let mut conditions = vec!["1 = 1".to_string()];
	let mut params: Vec<String> = Vec::new();

	if let Some(year) = input.year {
		conditions.push("strftime('%Y', date) = ?".to_string());
		params.push(year.to_string());
	}

	let where_clause = format!("WHERE {}", conditions.join(" AND "));

	let query = format!(
		"SELECT strftime('%Y-%m', date) as month,\n             SUM(CASE WHEN amount > 0 THEN amount ELSE 0 END) as income,\n             SUM(CASE WHEN amount < 0 AND category != 'Transfers' THEN -amount ELSE 0 END) as spending\n         FROM transactions\n         {}\n         GROUP BY month\n         ORDER BY month ASC",
		where_clause
	);

	let mut args = SqliteArguments::default();
	for param in params {
		let _ = args.add(param);
	}
	let rows = sqlx::query_with(&query, args).fetch_all(pool).await?;

	let mut labels = Vec::new();
	let mut income = Vec::new();
	let mut spending = Vec::new();

	for row in rows.iter() {
		let month: String = row.try_get("month").unwrap_or_else(|_| "Unknown".to_string());
		let income_value: f64 = row.try_get("income").unwrap_or(0.0);
		let spending_value: f64 = row.try_get("spending").unwrap_or(0.0);
		labels.push(month);
		income.push(income_value);
		spending.push(spending_value);
	}

	let summary = if labels.is_empty() {
		"No income/spending data found for the requested period.".to_string()
	} else {
		"Monthly income vs spending summary generated.".to_string()
	};

	let chart = ChartSpec {
		chart_type: "bar".to_string(),
		title: "Income vs Spending".to_string(),
		data: ChartData {
			labels,
			datasets: vec![
				Dataset {
					name: "Income".to_string(),
					values: income,
				},
				Dataset {
					name: "Spending".to_string(),
					values: spending,
				},
			],
		},
		height: Some(320),
	};

	Ok(ToolOutput {
		summary,
		charts: vec![chart],
	})
}

async fn search_transactions(
	pool: &SqlitePool,
	input: serde_json::Value,
) -> Result<ToolOutput, ToolError> {
	let input: TransactionSearchInput = serde_json::from_value(input)
		.map_err(|e| ToolError::InvalidInput(e.to_string()))?;

	let (conditions, params) = build_search_conditions(&input.search, &input.category, &input.year, &input.month);
	let where_clause = format!("WHERE {}", conditions.join(" AND "));

	// Summary query: total spend, count, avg, date range
	let summary_query = format!(
		"SELECT -SUM(amount) as total_spend, COUNT(*) as count, AVG(-amount) as avg_spend, MIN(date) as min_date, MAX(date) as max_date FROM transactions {}",
		where_clause
	);

	let mut args = SqliteArguments::default();
	for param in &params {
		let _ = args.add(param.clone());
	}
	let summary_row = sqlx::query_with(&summary_query, args).fetch_one(pool).await?;

	let total_spend: f64 = summary_row.try_get("total_spend").unwrap_or(0.0);
	let count: i64 = summary_row.try_get("count").unwrap_or(0);

	if count == 0 {
		return Ok(ToolOutput {
			summary: format!("No transactions found matching \"{}\".", input.search),
			charts: Vec::new(),
		});
	}

	let avg_spend: f64 = summary_row.try_get("avg_spend").unwrap_or(0.0);
	let min_date: String = summary_row.try_get("min_date").unwrap_or_else(|_| "?".to_string());
	let max_date: String = summary_row.try_get("max_date").unwrap_or_else(|_| "?".to_string());

	// Distinct merchant names
	let merchants_query = format!(
		"SELECT DISTINCT merchant_name FROM transactions {} ORDER BY merchant_name",
		where_clause
	);
	let mut args2 = SqliteArguments::default();
	for param in &params {
		let _ = args2.add(param.clone());
	}
	let merchant_rows = sqlx::query_with(&merchants_query, args2).fetch_all(pool).await?;
	let merchants: Vec<String> = merchant_rows.iter()
		.map(|row| row.try_get::<String, _>("merchant_name").unwrap_or_else(|_| "Unknown".to_string()))
		.collect();

	let summary = format!(
		"Search \"{}\": {} transactions, CHF {:.2} total, avg CHF {:.2}, range {} to {}. Merchant names: {}.",
		input.search, count, total_spend, avg_spend, min_date, max_date, merchants.join(", ")
	);

	// Monthly trend chart
	let trend_query = format!(
		"SELECT strftime('%Y-%m', date) as month, -SUM(amount) as spend FROM transactions {} GROUP BY month ORDER BY month ASC",
		where_clause
	);
	let mut args3 = SqliteArguments::default();
	for param in &params {
		let _ = args3.add(param.clone());
	}
	let trend_rows = sqlx::query_with(&trend_query, args3).fetch_all(pool).await?;

	let mut labels = Vec::new();
	let mut values = Vec::new();
	for row in trend_rows.iter() {
		let month: String = row.try_get("month").unwrap_or_else(|_| "Unknown".to_string());
		let spend: f64 = row.try_get("spend").unwrap_or(0.0);
		labels.push(month);
		values.push(spend);
	}

	let chart = ChartSpec {
		chart_type: "bar".to_string(),
		title: format!("\"{}\" Spending by Month", input.search),
		data: ChartData {
			labels,
			datasets: vec![Dataset {
				name: "CHF".to_string(),
				values,
			}],
		},
		height: Some(320),
	};

	Ok(ToolOutput {
		summary,
		charts: vec![chart],
	})
}

async fn list_transactions(
	pool: &SqlitePool,
	input: serde_json::Value,
) -> Result<ToolOutput, ToolError> {
	let input: ListTransactionsInput = serde_json::from_value(input)
		.map_err(|e| ToolError::InvalidInput(e.to_string()))?;
	let limit = input.limit.unwrap_or(50).max(1);

	let (conditions, params) = build_search_conditions(&input.search, &input.category, &input.year, &input.month);
	let where_clause = format!("WHERE {}", conditions.join(" AND "));

	// Get total count first
	let count_query = format!("SELECT COUNT(*) as total FROM transactions {}", where_clause);
	let mut count_args = SqliteArguments::default();
	for param in &params {
		let _ = count_args.add(param.clone());
	}
	let count_row = sqlx::query_with(&count_query, count_args).fetch_one(pool).await?;
	let total: i64 = count_row.try_get("total").unwrap_or(0);

	if total == 0 {
		return Ok(ToolOutput {
			summary: format!("No transactions found matching \"{}\".", input.search),
			charts: Vec::new(),
		});
	}

	// Fetch rows with limit
	let list_query = format!(
		"SELECT date, amount, merchant_name, raw_description FROM transactions {} ORDER BY date DESC LIMIT ?",
		where_clause
	);
	let mut list_args = SqliteArguments::default();
	for param in &params {
		let _ = list_args.add(param.clone());
	}
	let _ = list_args.add(limit);
	let rows = sqlx::query_with(&list_query, list_args).fetch_all(pool).await?;

	let mut lines = Vec::new();
	for row in rows.iter() {
		let date: String = row.try_get("date").unwrap_or_else(|_| "?".to_string());
		let amount: f64 = row.try_get("amount").unwrap_or(0.0);
		let merchant: String = row.try_get("merchant_name").unwrap_or_else(|_| "?".to_string());
		let raw: String = row.try_get("raw_description").unwrap_or_else(|_| "?".to_string());
		lines.push(format!("{} | CHF {:.2} | {} | {}", date, -amount, merchant, raw));
	}

	let shown = rows.len() as i64;
	let header = if shown < total {
		format!("Showing {} of {} transactions matching \"{}\":", shown, total, input.search)
	} else {
		format!("{} transactions matching \"{}\":", total, input.search)
	};

	let summary = format!("{}\n{}", header, lines.join("\n"));

	Ok(ToolOutput {
		summary,
		charts: Vec::new(),
	})
}
