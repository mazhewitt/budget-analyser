mod cache;
mod categories;
mod classifier;
mod csv_parser;
mod db;
mod review;

use std::path::Path;
use classifier::Classifier;
use db::{Database, CategoryInfo};
use review::{run_review, run_recategorise, ReviewFilters};

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
    categories: &[CategoryInfo],
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

    let examples = db.get_few_shot_examples()?;

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
            let res = classifier.classify(&tx.description, amount, &tx.details, &examples, categories);
            
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

    if args.len() < 2 {
        println!("Usage: budget-analyser <command> [args]");
        println!("Commands:");
        println!("  import <path> [db_path] [model] [endpoint]");
        println!("  review [db_path] [--category C] [--since S] [--until U] [--merchant M] [--threshold T]");
        println!("  reclassify [db_path] [model] [endpoint] [--category C] [--since S] [--until U] [--merchant M] [--threshold T]");
        println!("  recategorise --category <name> [db_path]");
        println!("  categories list [db_path]");
        println!("  categories add <name> <description> [db_path]");
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "import" => {
            let input_path = args.get(2).map(|s| s.as_str()).unwrap_or("data/synthetic-ubstransactions-feb2026.csv");
            let db_path = args.get(3).map(|s| s.as_str()).unwrap_or("data/budget.db");
            let model = args.get(4).map(|s| s.as_str()).unwrap_or("qwen3:8b");
            let endpoint = args.get(5).map(|s| s.as_str()).unwrap_or("http://localhost:11434");

            run_import(input_path, db_path, model, endpoint)
        }
        "review" => {
            let mut db_path = "data/budget.db";
            let mut category = None;
            let mut since = None;
            let mut until = None;
            let mut merchant = None;
            let mut threshold = 0.80;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--category" => { category = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--since" => { since = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--until" => { until = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--merchant" => { merchant = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--threshold" => { 
                        threshold = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0.80); 
                        i += 2; 
                    }
                    path if !path.starts_with("--") => {
                        db_path = path;
                        i += 1;
                    }
                    _ => i += 1,
                }
            }

            let db = Database::open(Path::new(db_path))?;
            let categories = db.list_categories()?;
            run_review(&db, ReviewFilters {
                category,
                since,
                until,
                merchant,
                threshold,
            }, &categories)
        }
        "reclassify" => {
            let mut db_path = "data/budget.db";
            let mut model = "qwen3:8b";
            let mut endpoint = "http://localhost:11434";
            let mut category = None;
            let mut since = None;
            let mut until = None;
            let mut merchant = None;
            let mut threshold = 0.80;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--category" => { category = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--since" => { since = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--until" => { until = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--merchant" => { merchant = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    "--threshold" => { 
                        threshold = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0.80); 
                        i += 2; 
                    }
                    path if !path.starts_with("--") => {
                        // Very basic heuristic: if it contains : it's a model or endpoint
                        if path.contains(":") {
                            if path.starts_with("http") {
                                endpoint = path;
                            } else {
                                model = path;
                            }
                        } else {
                            db_path = path;
                        }
                        i += 1;
                    }
                    _ => i += 1,
                }
            }

            run_reclassify(db_path, model, endpoint, ReviewFilters {
                category,
                since,
                until,
                merchant,
                threshold,
            })
        }
        "recategorise" => {
            let mut db_path = "data/budget.db";
            let mut category = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--category" => { category = args.get(i + 1).map(|s| s.as_str()); i += 2; }
                    path if !path.starts_with("--") => {
                        db_path = path;
                        i += 1;
                    }
                    _ => i += 1,
                }
            }

            if let Some(cat) = category {
                let db = Database::open(Path::new(db_path))?;
                let categories = db.list_categories()?;
                run_recategorise(&db, cat, &categories)
            } else {
                println!("Usage: budget-analyser recategorise --category <name> [db_path]");
                Ok(())
            }
        }
        "categories" => {
            let sub = args.get(2).map(|s| s.as_str()).unwrap_or("list");
            match sub {
                "list" => {
                    let db_path = args.get(3).map(|s| s.as_str()).unwrap_or("data/budget.db");
                    let db = Database::open(Path::new(db_path))?;
                    let cats = db.list_categories()?;
                    println!("Spending Categories:");
                    for cat in cats {
                        println!("- {}: {}", cat.name, cat.description);
                    }
                    Ok(())
                }
                "add" => {
                    let name = args.get(3).map(|s| s.as_str());
                    let desc = args.get(4).map(|s| s.as_str());
                    let db_path = args.get(5).map(|s| s.as_str()).unwrap_or("data/budget.db");
                    
                    if let (Some(n), Some(d)) = (name, desc) {
                        let db = Database::open(Path::new(db_path))?;
                        match db.add_category(n, d) {
                            Ok(()) => println!("Added category: {}", n),
                            Err(_) => println!("Error: category '{}' already exists.", n),
                        }
                        Ok(())
                    } else {
                        println!("Usage: budget-analyser categories add <name> <description> [db_path]");
                        Ok(())
                    }
                }
                _ => {
                    println!("Unknown categories subcommand: {}", sub);
                    Ok(())
                }
            }
        }
        // Backward compatibility
        path => {
            let db_path = args.get(2).map(|s| s.as_str()).unwrap_or("data/budget.db");
            let model = args.get(3).map(|s| s.as_str()).unwrap_or("qwen3:8b");
            let endpoint = args.get(4).map(|s| s.as_str()).unwrap_or("http://localhost:11434");

            run_import(path, db_path, model, endpoint)
        }
    }
}

fn run_reclassify(db_path: &str, model: &str, endpoint: &str, filters: ReviewFilters) -> Result<(), Box<dyn std::error::Error>> {
    println!("UBS Transaction Categoriser (Reclassify)");
    println!("  Database:   {}", db_path);
    println!("  Model:      {}", model);
    println!("  Endpoint:   {}", endpoint);
    println!();

    let db = Database::open(Path::new(db_path))?;
    let classifier = Classifier::new(endpoint, model);
    let examples = db.get_few_shot_examples()?;
    let categories = db.list_categories()?;

    let transactions = db.get_flagged_transactions(
        filters.threshold,
        filters.category,
        filters.since,
        filters.until,
        filters.merchant,
    )?;

    if transactions.is_empty() {
        println!("No transactions to reclassify.");
        return Ok(());
    }

    println!("Resetting and reclassifying {} transactions...", transactions.len());

    let mut cache_hits = 0;
    let mut llm_calls = 0;
    let mut category_changes = 0;

    for tx in transactions {
        let key = cache::normalise_merchant_key(&tx.raw_description);
        
        // Clear LLM cache entry to force re-evaluation
        db.delete_llm_cache_entry(&key)?;

        let result = if let Some(mut cached) = db.cache_lookup(&key)? {
            cache_hits += 1;
            cached.source = "cache".to_string();
            cached
        } else {
            llm_calls += 1;
            // Amount awareness: we don't have the original tx struct here, but we have amount
            let res = classifier.classify(&tx.raw_description, Some(tx.amount), "", &examples, &categories);
            
            // Store in cache
            db.cache_insert(&key, &res)?;
            res
        };

        if result.category != tx.category {
            category_changes += 1;
            println!("  {} → {} (was {})", tx.raw_description, result.category, tx.category);
        }

        db.update_transaction(tx.id, &result.merchant, &result.category, result.confidence, &result.source)?;
    }

    println!("\nReclassification Summary");
    println!("  Total processed:    {}", cache_hits + llm_calls);
    println!("  Cache hits:         {}", cache_hits);
    println!("  LLM calls:          {}", llm_calls);
    println!("  Category changes:   {}", category_changes);

    Ok(())
}

fn run_import(input_path: &str, db_path: &str, model: &str, endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("UBS Transaction Categoriser (Import)");
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
    let categories = db.list_categories()?;

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
        let file_stats = import_file(&db, &classifier, file_path, &categories)?;
        
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
