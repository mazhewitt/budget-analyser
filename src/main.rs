mod cache;
mod categories;
mod classifier;
mod csv_parser;
mod evaluator;

use std::path::Path;

use cache::{CacheEntry, MerchantCache};
use classifier::Classifier;
use evaluator::EvaluationInput;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let csv_path = args.get(1).map(|s| s.as_str()).unwrap_or("data/synthetic-ubstransactions-feb2026.csv");
    let model = args.get(2).map(|s| s.as_str()).unwrap_or("qwen3:8b");
    let endpoint = args.get(3).map(|s| s.as_str()).unwrap_or("http://localhost:11434");
    let threshold: f64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.80);
    let ground_truth_path = args.get(5).map(|s| s.as_str()).unwrap_or("data/ground-truth.toml");

    println!("Budget Analyser POC");
    println!("  CSV:        {}", csv_path);
    println!("  Model:      {}", model);
    println!("  Endpoint:   {}", endpoint);
    println!("  Threshold:  {:.2}", threshold);
    println!("  Ground truth: {}", ground_truth_path);
    println!();

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

    // Load ground truth
    let ground_truth = match evaluator::load_ground_truth(Path::new(ground_truth_path)) {
        Ok(gt) => {
            println!("Loaded {} ground-truth labels", gt.len());
            gt
        }
        Err(e) => {
            eprintln!("Error loading ground truth: {}", e);
            std::process::exit(1);
        }
    };

    // Classify each transaction
    let classifier = Classifier::new(endpoint, model);
    let mut cache = MerchantCache::new();
    let mut eval_inputs = Vec::new();

    for (i, tx) in transactions.iter().enumerate() {
        let result = if let Some(cached) = cache.lookup(&tx.description) {
            println!("  [{}/{}] {} → {} (cached)", i + 1, transactions.len(), tx.description, cached.category);
            classifier::ClassificationResult {
                merchant: cached.merchant.clone(),
                category: cached.category,
                confidence: cached.confidence,
            }
        } else {
            let amount = tx.debit.or(tx.credit);
            let result = classifier.classify(&tx.description, amount, &tx.details);
            println!(
                "  [{}/{}] {} → {} ({}) [{:.2}]",
                i + 1, transactions.len(), tx.description, result.category, result.merchant, result.confidence
            );
            cache.insert(&tx.description, CacheEntry {
                merchant: result.merchant.clone(),
                category: result.category,
                confidence: result.confidence,
            });
            result
        };

        eval_inputs.push(EvaluationInput {
            transaction_id: tx.transaction_id.clone(),
            description: tx.description.clone(),
            result,
        });
    }

    // Evaluate and print report
    let report = evaluator::evaluate(&eval_inputs, &ground_truth, threshold);
    evaluator::print_report(&report);
}
