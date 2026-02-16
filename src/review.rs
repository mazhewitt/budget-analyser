use std::io::{self, Write};
use std::collections::BTreeMap;
use crate::db::{Database, StoredTransaction, CategoryInfo};
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
    groups: usize,
    transactions: usize,
    confirmed: usize,
    corrected: usize,
    skipped: usize,
}

pub fn run_review(db: &Database, filters: ReviewFilters, categories: &[CategoryInfo]) -> Result<(), Box<dyn std::error::Error>> {
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

    // Group transactions by normalised merchant key (preserving insertion order via BTreeMap)
    let mut groups: BTreeMap<String, Vec<StoredTransaction>> = BTreeMap::new();
    for tx in transactions {
        let key = normalise_merchant_key(&tx.raw_description);
        groups.entry(key).or_default().push(tx);
    }

    let total_groups = groups.len();
    let total_txs: usize = groups.values().map(|g| g.len()).sum();
    println!("Starting review: {} transactions in {} groups", total_txs, total_groups);
    println!();

    let mut stats = ReviewStats {
        groups: 0,
        transactions: 0,
        confirmed: 0,
        corrected: 0,
        skipped: 0,
    };

    for (key, group) in &groups {
        stats.groups += 1;
        let first = &group[0];
        let total_amount: f64 = group.iter().map(|t| t.amount).sum();

        println!("=== Group {}/{}: {} ({} transactions) ===", stats.groups, total_groups, key, group.len());
        println!("Merchant: {}", first.merchant_name);
        println!("Category: {} (confidence: {:.2})", first.category, first.confidence);
        println!("Total:    {:.2} {}", total_amount, first.currency);

        for tx in group {
            println!("  {} | {:>10.2} | {}", tx.date, tx.amount, tx.raw_description);
        }
        println!();

        loop {
            print!("(1) Confirm all, (2) Category, (3) Merchant, (4) Skip, (5) Quit: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    let result = ClassificationResult {
                        merchant: first.merchant_name.clone(),
                        category: first.category.clone(),
                        confidence: 1.0,
                        source: "manual".to_string(),
                    };
                    db.cache_insert(&key, &result)?;
                    db.insert_few_shot_example(&key, &first.raw_description, &first.merchant_name, &first.category)?;

                    for tx in group {
                        db.update_transaction(tx.id, &first.merchant_name, &first.category, 1.0, "manual")?;
                    }
                    stats.confirmed += group.len();
                    stats.transactions += group.len();
                    println!("Confirmed {} transactions.", group.len());
                    break;
                }
                "2" => {
                    println!("\nSelect category:");
                    for (i, cat) in categories.iter().enumerate() {
                        println!("  {:2}. {}", i + 1, cat.name);
                    }
                    print!("Choice: ");
                    io::stdout().flush()?;
                    let mut cat_input = String::new();
                    io::stdin().read_line(&mut cat_input)?;
                    if let Ok(idx) = cat_input.trim().parse::<usize>() {
                        if idx > 0 && idx <= categories.len() {
                            let new_cat = &categories[idx - 1].name;
                            let result = ClassificationResult {
                                merchant: first.merchant_name.clone(),
                                category: new_cat.clone(),
                                confidence: 1.0,
                                source: "manual".to_string(),
                            };
                            db.cache_insert(&key, &result)?;
                            db.insert_few_shot_example(&key, &first.raw_description, &first.merchant_name, new_cat)?;

                            for tx in group {
                                db.update_transaction(tx.id, &first.merchant_name, new_cat, 1.0, "manual")?;
                            }
                            stats.corrected += group.len();
                            stats.transactions += group.len();
                            println!("Updated {} transactions to {}.", group.len(), new_cat);
                            break;
                        }
                    }
                    println!("Invalid category choice.");
                }
                "3" => {
                    print!("New merchant name: ");
                    io::stdout().flush()?;
                    let mut merchant_input = String::new();
                    io::stdin().read_line(&mut merchant_input)?;
                    let new_merchant = merchant_input.trim();
                    if !new_merchant.is_empty() {
                        let result = ClassificationResult {
                            merchant: new_merchant.to_string(),
                            category: first.category.clone(),
                            confidence: 1.0,
                            source: "manual".to_string(),
                        };
                        db.cache_insert(&key, &result)?;
                        db.insert_few_shot_example(&key, &first.raw_description, new_merchant, &first.category)?;

                        for tx in group {
                            db.update_transaction(tx.id, new_merchant, &first.category, 1.0, "manual")?;
                        }
                        stats.corrected += group.len();
                        stats.transactions += group.len();
                        println!("Updated {} transactions to merchant {}.", group.len(), new_merchant);
                        break;
                    }
                    println!("Merchant name cannot be empty.");
                }
                "4" => {
                    stats.skipped += group.len();
                    stats.transactions += group.len();
                    println!("Skipped {} transactions.", group.len());
                    break;
                }
                "5" | "q" | "quit" => {
                    print_summary(&stats);
                    return Ok(());
                }
                _ => println!("Invalid choice."),
            }
        }
        println!();
    }

    print_summary(&stats);
    Ok(())
}

pub fn run_recategorise(db: &Database, category: &str, categories: &[CategoryInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let transactions = db.get_transactions_by_category(category)?;

    if transactions.is_empty() {
        println!("No transactions found in category '{}'.", category);
        return Ok(());
    }

    // Group by normalised merchant key
    let mut groups: BTreeMap<String, Vec<StoredTransaction>> = BTreeMap::new();
    for tx in transactions {
        let key = normalise_merchant_key(&tx.raw_description);
        groups.entry(key).or_default().push(tx);
    }

    let total_groups = groups.len();
    let total_txs: usize = groups.values().map(|g| g.len()).sum();
    println!("Recategorising {}: {} transactions in {} groups", category, total_txs, total_groups);
    println!();

    let mut stats = ReviewStats {
        groups: 0,
        transactions: 0,
        confirmed: 0,
        corrected: 0,
        skipped: 0,
    };

    for (key, group) in &groups {
        stats.groups += 1;
        let first = &group[0];
        let total_amount: f64 = group.iter().map(|t| t.amount).sum();

        println!("=== Group {}/{}: {} ({} transactions) ===", stats.groups, total_groups, key, group.len());
        println!("Merchant: {}", first.merchant_name);
        println!("Category: {}", first.category);
        println!("Total:    {:.2} {}", total_amount, first.currency);

        for tx in group {
            println!("  {} | {:>10.2} | {}", tx.date, tx.amount, tx.raw_description);
        }
        println!();

        loop {
            print!("(1) Change category, (2) Skip, (3) Quit: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    println!("\nSelect category:");
                    for (i, cat) in categories.iter().enumerate() {
                        println!("  {:2}. {}", i + 1, cat.name);
                    }
                    print!("Choice: ");
                    io::stdout().flush()?;
                    let mut cat_input = String::new();
                    io::stdin().read_line(&mut cat_input)?;
                    if let Ok(idx) = cat_input.trim().parse::<usize>() {
                        if idx > 0 && idx <= categories.len() {
                            let new_cat = &categories[idx - 1].name;
                            let result = ClassificationResult {
                                merchant: first.merchant_name.clone(),
                                category: new_cat.clone(),
                                confidence: 1.0,
                                source: "manual".to_string(),
                            };
                            db.cache_insert(&key, &result)?;
                            db.insert_few_shot_example(&key, &first.raw_description, &first.merchant_name, new_cat)?;

                            for tx in group {
                                db.update_transaction(tx.id, &first.merchant_name, new_cat, 1.0, "manual")?;
                            }
                            stats.corrected += group.len();
                            stats.transactions += group.len();
                            println!("Updated {} transactions to {}.", group.len(), new_cat);
                            break;
                        }
                    }
                    println!("Invalid category choice.");
                }
                "2" => {
                    stats.skipped += group.len();
                    stats.transactions += group.len();
                    println!("Skipped {} transactions.", group.len());
                    break;
                }
                "3" | "q" | "quit" => {
                    print_recat_summary(&stats);
                    return Ok(());
                }
                _ => println!("Invalid choice."),
            }
        }
        println!();
    }

    print_recat_summary(&stats);
    Ok(())
}

fn print_summary(stats: &ReviewStats) {
    println!("\nReview complete.");
    println!("  Groups reviewed: {}", stats.groups);
    println!("  Transactions:    {}", stats.transactions);
    println!("  Confirmed:       {}", stats.confirmed);
    println!("  Corrected:       {}", stats.corrected);
    println!("  Skipped:         {}", stats.skipped);
}

fn print_recat_summary(stats: &ReviewStats) {
    println!("\nRecategorise complete.");
    println!("  Groups reviewed: {}", stats.groups);
    println!("  Transactions:    {}", stats.transactions);
    println!("  Updated:         {}", stats.corrected);
    println!("  Skipped:         {}", stats.skipped);
}
