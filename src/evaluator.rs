use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::classifier::ClassificationResult;

#[derive(Debug, Deserialize)]
struct GroundTruthFile {
    labels: HashMap<String, GroundTruthEntry>,
}

#[derive(Debug, Deserialize)]
pub struct GroundTruthEntry {
    pub merchant: String,
    pub category: String,
}

pub struct EvaluationInput {
    pub transaction_id: String,
    pub description: String,
    pub result: ClassificationResult,
}

pub struct EvaluationReport {
    pub total: usize,
    pub overall_correct: usize,
    pub above_threshold: usize,
    pub above_threshold_correct: usize,
    pub below_threshold: usize,
    pub false_confidence: Vec<FalseConfidenceEntry>,
    pub misclassified: Vec<MisclassifiedEntry>,
    pub per_category: HashMap<String, CategoryStats>,
}

#[derive(Debug)]
pub struct FalseConfidenceEntry {
    pub transaction_id: String,
    pub description: String,
    pub predicted: String,
    pub expected: String,
    pub confidence: f64,
}

#[derive(Debug)]
pub struct MisclassifiedEntry {
    pub transaction_id: String,
    pub description: String,
    pub predicted: String,
    pub expected: String,
    pub confidence: f64,
}

#[derive(Debug, Default)]
pub struct CategoryStats {
    pub total: usize,
    pub correct: usize,
}

pub fn load_ground_truth(path: &Path) -> Result<HashMap<String, GroundTruthEntry>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read ground-truth file '{}': {}", path.display(), e))?;
    let file: GroundTruthFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse ground-truth TOML: {}", e))?;
    Ok(file.labels)
}

pub fn evaluate(
    inputs: &[EvaluationInput],
    ground_truth: &HashMap<String, GroundTruthEntry>,
    confidence_threshold: f64,
) -> EvaluationReport {
    let mut report = EvaluationReport {
        total: inputs.len(),
        overall_correct: 0,
        above_threshold: 0,
        above_threshold_correct: 0,
        below_threshold: 0,
        false_confidence: Vec::new(),
        misclassified: Vec::new(),
        per_category: HashMap::new(),
    };

    for input in inputs {
        let expected = match ground_truth.get(&input.transaction_id) {
            Some(gt) => gt,
            None => {
                eprintln!("Warning: no ground-truth label for transaction {}", input.transaction_id);
                continue;
            }
        };

        let predicted_str = format!("{:?}", input.result.category);
        let is_correct = predicted_str == expected.category;

        // Per-category stats
        let stats = report.per_category
            .entry(expected.category.clone())
            .or_default();
        stats.total += 1;
        if is_correct {
            stats.correct += 1;
            report.overall_correct += 1;
        }

        let above = input.result.confidence >= confidence_threshold;

        if above {
            report.above_threshold += 1;
            if is_correct {
                report.above_threshold_correct += 1;
            } else {
                report.false_confidence.push(FalseConfidenceEntry {
                    transaction_id: input.transaction_id.clone(),
                    description: input.description.clone(),
                    predicted: predicted_str.clone(),
                    expected: expected.category.clone(),
                    confidence: input.result.confidence,
                });
            }
        } else {
            report.below_threshold += 1;
        }

        if !is_correct {
            report.misclassified.push(MisclassifiedEntry {
                transaction_id: input.transaction_id.clone(),
                description: input.description.clone(),
                predicted: predicted_str,
                expected: expected.category.clone(),
                confidence: input.result.confidence,
            });
        }
    }

    report
}

pub fn print_report(report: &EvaluationReport) {
    println!("\n=== POC Evaluation Report ===\n");

    let overall_accuracy = if report.total > 0 {
        100.0 * report.overall_correct as f64 / report.total as f64
    } else {
        0.0
    };

    let auto_accepted_accuracy = if report.above_threshold > 0 {
        100.0 * report.above_threshold_correct as f64 / report.above_threshold as f64
    } else {
        0.0
    };

    let flagging_rate = if report.total > 0 {
        100.0 * report.below_threshold as f64 / report.total as f64
    } else {
        0.0
    };

    println!("Total transactions:       {}", report.total);
    println!("Overall accuracy:         {:.1}% ({}/{})", overall_accuracy, report.overall_correct, report.total);
    println!("Auto-accepted accuracy:   {:.1}% ({}/{})", auto_accepted_accuracy, report.above_threshold_correct, report.above_threshold);
    println!("Flagging rate:            {:.1}% ({}/{})", flagging_rate, report.below_threshold, report.total);
    println!("False-confidence results: {}", report.false_confidence.len());

    println!("\n--- Per-Category Breakdown ---\n");
    let mut categories: Vec<_> = report.per_category.iter().collect();
    categories.sort_by_key(|(name, _)| (*name).clone());
    for (name, stats) in &categories {
        let acc = if stats.total > 0 {
            100.0 * stats.correct as f64 / stats.total as f64
        } else {
            0.0
        };
        println!("  {:<20} {:.1}% ({}/{})", name, acc, stats.correct, stats.total);
    }

    if !report.false_confidence.is_empty() {
        println!("\n--- False-Confidence Results (HIGH confidence, WRONG category) ---\n");
        for fc in &report.false_confidence {
            println!("  [{}] \"{}\"", fc.transaction_id, fc.description);
            println!("    Predicted: {} (confidence: {:.2})", fc.predicted, fc.confidence);
            println!("    Expected:  {}", fc.expected);
            println!();
        }
    }

    if !report.misclassified.is_empty() {
        println!("\n--- All Misclassified Transactions ---\n");
        for m in &report.misclassified {
            println!("  [{}] \"{}\"", m.transaction_id, m.description);
            println!("    Predicted: {} (confidence: {:.2})", m.predicted, m.confidence);
            println!("    Expected:  {}", m.expected);
            println!();
        }
    }

    println!("=== End of Report ===");
}
