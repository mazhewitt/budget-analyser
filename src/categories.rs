use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Groceries,
    Transport,
    Utilities,
    Subscriptions,
    Transfers,
    CardPayments,
    Insurance,
    Taxes,
    Shopping,
    Dining,
    Income,
    Uncategorised,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Groceries => write!(f, "Groceries"),
            Category::Transport => write!(f, "Transport"),
            Category::Utilities => write!(f, "Utilities"),
            Category::Subscriptions => write!(f, "Subscriptions"),
            Category::Transfers => write!(f, "Transfers"),
            Category::CardPayments => write!(f, "Card Payments"),
            Category::Insurance => write!(f, "Insurance"),
            Category::Taxes => write!(f, "Taxes"),
            Category::Shopping => write!(f, "Shopping"),
            Category::Dining => write!(f, "Dining"),
            Category::Income => write!(f, "Income"),
            Category::Uncategorised => write!(f, "Uncategorised"),
        }
    }
}

impl Category {
    pub fn description(&self) -> &'static str {
        match self {
            Category::Groceries => "Supermarkets and food shopping (Migros, Coop, Aldi, Lidl)",
            Category::Transport => "Public transport, taxis, fuel, parking (SBB, TWINT transport, EasyRide)",
            Category::Utilities => "Electricity, water, gas, internet, phone (Sunrise, Swisscom, EWZ)",
            Category::Subscriptions => "Recurring subscriptions and memberships (streaming, software, gym)",
            Category::Transfers => "Personal transfers between accounts or to other people",
            Category::CardPayments => "Aggregated credit/debit card statement postings",
            Category::Insurance => "Health, life, home, and liability insurance premiums (Visana, Vivao Sympa)",
            Category::Taxes => "Tax payments to cantonal or federal authorities (Steuerverwaltung)",
            Category::Shopping => "General retail purchases, electronics, clothing (Digitec, Zalando)",
            Category::Dining => "Restaurants, cafés, takeaway, food delivery",
            Category::Income => "Incoming salary, payments, refunds, credits",
            Category::Uncategorised => "Transactions that could not be confidently classified",
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Category::Groceries,
            Category::Transport,
            Category::Utilities,
            Category::Subscriptions,
            Category::Transfers,
            Category::CardPayments,
            Category::Insurance,
            Category::Taxes,
            Category::Shopping,
            Category::Dining,
            Category::Income,
            Category::Uncategorised,
        ]
    }

    pub fn schema_for_prompt() -> String {
        let mut out = String::new();
        for cat in Self::all() {
            out.push_str(&format!("- {}: {}\n", cat, cat.description()));
        }
        out
    }
}
