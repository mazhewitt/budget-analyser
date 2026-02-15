mod cache;
mod categories;
mod classifier;
mod csv_parser;
mod db;

use std::path::Path;
use classifier::Classifier;
use db::Database;

#[derive(Default, Debug)]
struct ImportStats {
    total_parsed: usize,
    new_insertions: usize,
    duplicates_skipped: usize,
    cache_hits: usize,
    llm_calls: usize,
}

impl ImportStats {
    fn accumulate(&mut self, other: &ImportStats) {
        self.total_parsed += other.total_parsed;
        self.new_insertions += other.new_insertions;
        self.duplicates_skipped += other.duplicates_skipped;
        self.cache_hits += other.cache_hits;
        self.llm_calls += other.llm_calls;
    }
}

fn import_file(
    db: &Database,
    classifier: &Classifier,
    csv_path: &Path,
) -> Result<ImportStats, Box<dyn std::error::Error>> {
    let transactions = csv_parser::parse_csv(csv_path)?;
    let total = transactions.len();
    
    let mut stats = ImportStats {
        total_parsed: total,
        ..Default::default()
    };

    let import_batch = csv_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    for (i, tx) in transactions.iter().enumerate() {
        // 1. Duplicate detection
        if db.transaction_exists(&tx.transaction_id)? {
            stats.duplicates_skipped += 1;
            continue;
        }

        // 2. Normalise key for cache lookup
        let key = cache::normalise_merchant_key(&tx.description);

        // 3. Cache lookup vs LLM classification
        let result = if let Some(mut cached) = db.cache_lookup(&key)? {
            stats.cache_hits += 1;
            cached.source = "cache".to_string();
            cached
        } else {
            stats.llm_calls += 1;
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
            stats.new_insertions += 1;
        }
    }

    // 5. Log import run
    db.log_import(import_batch, stats.new_insertions)?;

    Ok(stats)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let input_path = args.get(1).map(|s| s.as_str()).unwrap_or("data/synthetic-ubstransactions-feb2026.csv");
    let db_path = args.get(2).map(|s| s.as_str()).unwrap_or("data/budget.db");
    let model = args.get(3).map(|s| s.as_str()).unwrap_or("qwen3:8b");
    let endpoint = args.get(4).map(|s| s.as_str()).unwrap_or("http://localhost:11434");

    println!("UBS Transaction Categoriser");
    println!("  Input:      {}", input_path);
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
    let classifier = Classifier::new(endpoint, model);

    let metadata = std::fs::metadata(input_path)?;
    let mut files = Vec::new();

    if metadata.is_dir() {
        for entry in std::fs::read_dir(input_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                files.push(path);
            }
        }
        files.sort();
    } else {
        files.push(Path::new(input_path).to_path_buf());
    }

    if files.is_empty() {
        println!("No CSV files found at: {}", input_path);
        return Ok(());
    }

    let mut overall_stats = ImportStats::default();
    let total_files = files.len();

    for (i, file_path) in files.iter().enumerate() {
        println!("Importing file {}/{}: {}", i + 1, total_files, file_path.display());
        let file_stats = import_file(&db, &classifier, file_path)?;
        
        println!("  File Summary: {} parsed, {} new, {} skipped, {} cache hits, {} llm calls",
            file_stats.total_parsed,
            file_stats.new_insertions,
            file_stats.duplicates_skipped,
            file_stats.cache_hits,
            file_stats.llm_calls
        );
        println!();
        
        overall_stats.accumulate(&file_stats);
    }

    if total_files > 1 {
        println!("Overall Summary ({} files)", total_files);
        println!("  Total parsed:       {}", overall_stats.total_parsed);
        println!("  Total new:          {}", overall_stats.new_insertions);
        println!("  Total skipped:      {}", overall_stats.duplicates_skipped);
        println!("  Total cache hits:   {}", overall_stats.cache_hits);
        println!("  Total LLM calls:    {}", overall_stats.llm_calls);
    } else if total_files == 1 {
        println!("Import Complete");
        println!("  Total parsed:       {}", overall_stats.total_parsed);
        println!("  New insertions:     {}", overall_stats.new_insertions);
        println!("  Duplicates skipped: {}", overall_stats.duplicates_skipped);
        println!("  Cache hits:         {}", overall_stats.cache_hits);
        println!("  LLM calls:          {}", overall_stats.llm_calls);
    }

    Ok(())
}
