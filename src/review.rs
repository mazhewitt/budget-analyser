use std::io::{self, Write};
use crate::db::Database;
use crate::categories::Category;
use crate::cache::normalise_merchant_key;
use crate::classifier::ClassificationResult;

pub struct ReviewFilters<'a> {
    pub category: Option<&'a str>,
    pub since: Option<&'a str>,
    pub until: Option<&'a str>,
    pub merchant: Option<&'a str>,
    pub threshold: f64,
}

struct ReviewStats {
    total: usize,
    confirmed: usize,
    corrected: usize,
    skipped: usize,
}

pub fn run_review(db: &Database, filters: ReviewFilters) -> Result<(), Box<dyn std::error::Error>> {
    let transactions = db.get_flagged_transactions(
        filters.threshold,
        filters.category,
        filters.since,
        filters.until,
        filters.merchant,
    )?;

    if transactions.is_empty() {
        println!("No transactions to review.");
        return Ok(());
    }

    let mut stats = ReviewStats {
        total: 0,
        confirmed: 0,
        corrected: 0,
        skipped: 0,
    };

    let total_flagged = transactions.len();
    println!("Starting review of {} flagged transactions...", total_flagged);
    println!();

    for tx in transactions {
        stats.total += 1;
        println!("--- Transaction {}/{} ---", stats.total, total_flagged);
        println!("Date:     {}", tx.date);
        println!("Amount:   {:.2} {}", tx.amount, tx.currency);
        println!("Raw:      {}", tx.raw_description);
        println!("Merchant: {}", tx.merchant_name);
        println!("Category: {} (confidence: {:.2})", tx.category, tx.confidence);
        println!();

        loop {
            print!("(1) Confirm, (2) Category, (3) Merchant, (4) Skip, (5) Quit: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    // Confirm
                    db.update_transaction(tx.id, &tx.merchant_name, &tx.category, 1.0, "manual")?;
                    let key = normalise_merchant_key(&tx.raw_description);
                    db.cache_insert(&key, &ClassificationResult {
                        merchant: tx.merchant_name.clone(),
                        category: serde_json::from_value(serde_json::Value::String(tx.category.clone()))
                            .unwrap_or(Category::Uncategorised),
                        confidence: 1.0,
                        source: "manual".to_string(),
                    })?;
                    stats.confirmed += 1;
                    println!("Confirmed.");
                    break;
                }
                "2" => {
                    // Change category
                    let categories = Category::all();
                    println!("
Select category:");
                    for (i, cat) in categories.iter().enumerate() {
                        println!("  {:2}. {}", i + 1, cat);
                    }
                    print!("Choice: ");
                    io::stdout().flush()?;
                    let mut cat_input = String::new();
                    io::stdin().read_line(&mut cat_input)?;
                    if let Ok(idx) = cat_input.trim().parse::<usize>() {
                        if idx > 0 && idx <= categories.len() {
                            let new_cat = categories[idx - 1];
                            db.update_transaction(tx.id, &tx.merchant_name, &new_cat.to_string(), 1.0, "manual")?;
                            let key = normalise_merchant_key(&tx.raw_description);
                            db.cache_insert(&key, &ClassificationResult {
                                merchant: tx.merchant_name.clone(),
                                category: new_cat,
                                confidence: 1.0,
                                source: "manual".to_string(),
                            })?;
                            stats.corrected += 1;
                            println!("Updated category to {}.", new_cat);
                            break;
                        }
                    }
                    println!("Invalid category choice.");
                }
                "3" => {
                    // Edit merchant
                    print!("New merchant name: ");
                    io::stdout().flush()?;
                    let mut merchant_input = String::new();
                    io::stdin().read_line(&mut merchant_input)?;
                    let new_merchant = merchant_input.trim();
                    if !new_merchant.is_empty() {
                        db.update_transaction(tx.id, new_merchant, &tx.category, 1.0, "manual")?;
                        let key = normalise_merchant_key(&tx.raw_description);
                        db.cache_insert(&key, &ClassificationResult {
                            merchant: new_merchant.to_string(),
                            category: serde_json::from_value(serde_json::Value::String(tx.category.clone()))
                                .unwrap_or(Category::Uncategorised),
                            confidence: 1.0,
                            source: "manual".to_string(),
                        })?;
                        stats.corrected += 1;
                        println!("Updated merchant to {}.", new_merchant);
                        break;
                    }
                    println!("Merchant name cannot be empty.");
                }
                "4" => {
                    // Skip
                    stats.skipped += 1;
                    println!("Skipped.");
                    break;
                }
                "5" | "q" | "quit" => {
                    println!("
Review session summary:");
                    println!("  Reviewed:  {}", stats.total);
                    println!("  Confirmed: {}", stats.confirmed);
                    println!("  Corrected: {}", stats.corrected);
                    println!("  Skipped:   {}", stats.skipped);
                    return Ok(());
                }
                _ => println!("Invalid choice."),
            }
        }
        println!();
    }

    println!("Review complete.");
    println!("  Reviewed:  {}", stats.total);
    println!("  Confirmed: {}", stats.confirmed);
    println!("  Corrected: {}", stats.corrected);
    println!("  Skipped:   {}", stats.skipped);

    Ok(())
}
