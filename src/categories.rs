use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Groceries,
    Dining,
    Transport,
    Housing,
    Insurance,
    Healthcare,
    Shopping,
    Subscriptions,
    Children,
    Travel,
    Cash,
    Transfers,
    Income,
    Fees,
    Other,
    Uncategorised,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Groceries => write!(f, "Groceries"),
            Category::Dining => write!(f, "Dining"),
            Category::Transport => write!(f, "Transport"),
            Category::Housing => write!(f, "Housing"),
            Category::Insurance => write!(f, "Insurance"),
            Category::Healthcare => write!(f, "Healthcare"),
            Category::Shopping => write!(f, "Shopping"),
            Category::Subscriptions => write!(f, "Subscriptions"),
            Category::Children => write!(f, "Children"),
            Category::Travel => write!(f, "Travel"),
            Category::Cash => write!(f, "Cash"),
            Category::Transfers => write!(f, "Transfers"),
            Category::Income => write!(f, "Income"),
            Category::Fees => write!(f, "Fees"),
            Category::Other => write!(f, "Other"),
            Category::Uncategorised => write!(f, "Uncategorised"),
        }
    }
}

impl Category {
    pub fn description(&self) -> &'static str {
        match self {
            Category::Groceries => "Supermarkets, food shops, bakeries, butchers",
            Category::Dining => "Restaurants, cafes, bars, takeaway, fast food",
            Category::Transport => "Public transport, taxis, fuel, parking, car expenses",
            Category::Housing => "Rent, mortgage, utilities, electricity, water, heating",
            Category::Insurance => "Health insurance, liability, household, car insurance",
            Category::Healthcare => "Doctors, dentists, pharmacy, hospital, optician",
            Category::Shopping => "Clothing, electronics, furniture, household goods",
            Category::Subscriptions => "Streaming, software, newspapers, memberships, phone plan",
            Category::Children => "Childcare, school, activities, toys, children's clothing",
            Category::Travel => "Hotels, flights, holiday expenses",
            Category::Cash => "ATM withdrawals",
            Category::Transfers => "Transfers between own accounts, savings",
            Category::Income => "Salary, refunds, reimbursements",
            Category::Fees => "Bank fees, card fees, foreign exchange fees",
            Category::Other => "Anything that doesn't fit above",
            Category::Uncategorised => "Transactions that could not be confidently classified",
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Category::Groceries,
            Category::Dining,
            Category::Transport,
            Category::Housing,
            Category::Insurance,
            Category::Healthcare,
            Category::Shopping,
            Category::Subscriptions,
            Category::Children,
            Category::Travel,
            Category::Cash,
            Category::Transfers,
            Category::Income,
            Category::Fees,
            Category::Other,
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
