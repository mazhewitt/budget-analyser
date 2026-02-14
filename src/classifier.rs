use serde::{Deserialize, Serialize};

use crate::categories::Category;

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub merchant: String,
    pub category: Category,
    pub confidence: f64,
}

pub struct Classifier {
    client: reqwest::blocking::Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    format: &'static str,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct LlmOutput {
    merchant: Option<String>,
    category: Option<String>,
    confidence: Option<f64>,
}

impl Classifier {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    pub fn classify(&self, description: &str, amount: Option<f64>, details: &str) -> ClassificationResult {
        let system_prompt = Self::build_system_prompt();
        let user_prompt = Self::build_user_prompt(description, amount, details);

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message { role: "system", content: system_prompt },
                Message { role: "user", content: user_prompt },
            ],
            format: "json",
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        let response = match self.client.post(&url).json(&request).send() {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Ollama request failed: {}", e);
                return Self::fallback(description);
            }
        };

        let chat_resp: ChatResponse = match response.json() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to parse Ollama response: {}", e);
                return Self::fallback(description);
            }
        };

        Self::parse_llm_output(&chat_resp.message.content, description)
    }

    fn build_system_prompt() -> String {
        format!(
            r#"You are a Swiss bank transaction classifier. Given a transaction description from a UBS bank statement, extract the merchant name and assign a spending category.

Categories:
{}
Respond with a JSON object containing exactly these fields:
- "merchant": the normalised merchant name (human-readable, e.g. "SBB" not "SBB MOBILE 9992402GK6077402")
- "category": one of the category names listed above (exactly as written, e.g. "Transport" not "transport")
- "confidence": a float between 0.0 and 1.0 indicating how confident you are in the classification

Examples of UBS merchant strings and their classifications:
- "SBB MOBILE" → {{"merchant": "SBB", "category": "Transport", "confidence": 0.95}}
- "SBB EASYRIDE" → {{"merchant": "SBB EasyRide", "category": "Transport", "confidence": 0.95}}
- "Steuerverwaltung EBILL-RECHT" → {{"merchant": "Steuerverwaltung", "category": "Taxes", "confidence": 0.90}}
- "MIGROS BASEL M 01234 KARTE 1234" → {{"merchant": "Migros Basel", "category": "Groceries", "confidence": 0.95}}
- "Vivao Sympa EBILL-RECHT" → {{"merchant": "Vivao Sympa", "category": "Insurance", "confidence": 0.85}}
- "Visana Service EBILL-RECHT" → {{"merchant": "Visana", "category": "Insurance", "confidence": 0.90}}
- "DIGITEC GALAXUS" → {{"merchant": "Digitec Galaxus", "category": "Shopping", "confidence": 0.95}}
- "UBS Switzerland" with details "CREDIT CARD STATEMENT" → {{"merchant": "UBS", "category": "CardPayments", "confidence": 0.95}}
- "Bob" with details "Debit UBS TWINT" → {{"merchant": "Bob", "category": "Transfers", "confidence": 0.80}}"#,
            Category::schema_for_prompt()
        )
    }

    fn build_user_prompt(description: &str, amount: Option<f64>, details: &str) -> String {
        let mut prompt = format!("Transaction: {}", description);
        if let Some(amt) = amount {
            prompt.push_str(&format!("\nAmount: CHF {:.2}", amt.abs()));
        }
        if !details.is_empty() {
            prompt.push_str(&format!("\nDetails: {}", details));
        }
        prompt
    }

    fn parse_llm_output(content: &str, description: &str) -> ClassificationResult {
        let parsed: LlmOutput = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse LLM JSON output: {} — raw: {}", e, content);
                return Self::fallback(description);
            }
        };

        let category = parsed
            .category
            .as_deref()
            .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
            .unwrap_or(Category::Uncategorised);

        ClassificationResult {
            merchant: parsed.merchant.unwrap_or_else(|| description.to_string()),
            category,
            confidence: parsed.confidence.unwrap_or(0.0),
        }
    }

    fn fallback(description: &str) -> ClassificationResult {
        ClassificationResult {
            merchant: description.to_string(),
            category: Category::Uncategorised,
            confidence: 0.0,
        }
    }
}
