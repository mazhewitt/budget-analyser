mod cache;
mod categories;
mod classifier;
mod csv_parser;
mod db;

use std::path::Path;
use classifier::Classifier;
use db::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let csv_path = args.get(1).map(|s| s.as_str()).unwrap_or("data/synthetic-ubstransactions-feb2026.csv");
    let db_path = args.get(2).map(|s| s.as_str()).unwrap_or("data/budget.db");
    let model = args.get(3).map(|s| s.as_str()).unwrap_or("qwen3:8b");
    let endpoint = args.get(4).map(|s| s.as_str()).unwrap_or("http://localhost:11434");

    println!("UBS Transaction Categoriser");
    println!("  CSV:        {}", csv_path);
    println!("  Database:   {}", db_path);
    println!("  Model:      {}", model);
    println!("  Endpoint:   {}", endpoint);
    println!();

    // Ensure data directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Open database
    let db = Database::open(Path::new(db_path))?;

    // Parse CSV
    let transactions = match csv_parser::parse_csv(Path::new(csv_path)) {
        Ok(txs) => {
            println!("Parsed {} transactions from CSV", txs.len());
            txs
        }
        Err(e) => {
            eprintln!("Error parsing CSV: {}", e);
            std::process::exit(1);
        }
    };

    let classifier = Classifier::new(endpoint, model);
    let mut new_insertions = 0;
    let mut duplicates_skipped = 0;
    let mut cache_hits = 0;
    let mut llm_calls = 0;

    let total = transactions.len();
    let import_batch = Path::new(csv_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    for (i, tx) in transactions.iter().enumerate() {
        // 1. Duplicate detection
        if db.transaction_exists(&tx.transaction_id)? {
            duplicates_skipped += 1;
            continue;
        }

        // 2. Normalise key for cache lookup
        let key = cache::normalise_merchant_key(&tx.description);

        // 3. Cache lookup vs LLM classification
        let result = if let Some(mut cached) = db.cache_lookup(&key)? {
            cache_hits += 1;
            cached.source = "cache".to_string();
            cached
        } else {
            llm_calls += 1;
            let amount = tx.debit.or(tx.credit);
            let res = classifier.classify(&tx.description, amount, &tx.details);
            
            // Store in cache for future use
            db.cache_insert(&key, &res)?;
            res
        };

        println!(
            "  [{}/{}] {} → {} ({}) [{:.2}] via {}",
            i + 1, total, tx.description, result.category, result.merchant, result.confidence, result.source
        );

        // 4. Insert transaction
        if db.insert_transaction(tx, &result, Some(import_batch))? {
            new_insertions += 1;
        }
    }

    // 5. Log import run
    db.log_import(import_batch, new_insertions)?;

    // 6. Print summary
    println!("\nImport Summary");
    println!("  Total parsed:       {}", total);
    println!("  New insertions:     {}", new_insertions);
    println!("  Duplicates skipped: {}", duplicates_skipped);
    println!("  Cache hits:         {}", cache_hits);
    println!("  LLM calls:          {}", llm_calls);

    Ok(())
}
