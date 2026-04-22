use std::collections::HashMap;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::classifier::ClassificationResult;

/// Rules-based classifier for UBS credit card transactions.
/// 
/// Precedence order:
/// 1. Merchant overrides (regex patterns)
/// 2. Sector lookup (MCC mapping)
/// 3. None (fall back to LLM)
/// 
/// Authoritative sector mapping reference:
/// openspec/specs/rules-based-cc-classification/spec.md
pub fn classify(description: &str, sector: Option<&str>) -> Option<ClassificationResult> {
    // 1. Merchant Overrides
    for (re, merchant_label, category) in OVERRIDES.iter() {
        if re.is_match(description) {
            let merchant = if *merchant_label == "Normalise" {
                normalise_merchant(description)
            } else {
                merchant_label.to_string()
            };
            return Some(ClassificationResult {
                merchant,
                category: category.to_string(),
                confidence: 0.95,
                source: "rules".to_string(),
            });
        }
    }

    // 2. Sector Lookup
    if let Some(sec) = sector {
        let sec_trimmed = sec.trim();
        if !sec_trimmed.is_empty() {
            if let Some(category) = SECTOR_MAP.get(sec_trimmed) {
                return Some(ClassificationResult {
                    merchant: normalise_merchant(description),
                    category: category.to_string(),
                    confidence: 0.90,
                    source: "rules".to_string(),
                });
            } else {
                tracing::warn!("unmapped CC sector: {}", sec_trimmed);
            }
        }
    }

    None
}

pub fn normalise_merchant(description: &str) -> String {
    let mut s = description.to_string();

    // 1. Strip trailing ISO-3 country codes preceded by whitespace
    static RE_COUNTRY: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+[A-Z]{3}$").unwrap());
    s = RE_COUNTRY.replace(&s, "").to_string();

    // 2. Strip trailing location tails (repeated whitespace followed by a city name or postcode pattern)
    // We'll strip things that look like postcodes (4-5 digits) or ZH 2
    static RE_LOCATION: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{2,}(\d{4,5}|[A-Z]{2}\s+\d)$").unwrap());
    s = RE_LOCATION.replace(&s, "").to_string();

    // 3. Collapse runs of 2+ whitespace characters to a single space
    static RE_WS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{2,}").unwrap());
    s = RE_WS.replace_all(&s, " ").to_string();

    // 4. Strip trailing card-number masks and everything after
    static RE_MASK: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\*X]{3,}.*$").unwrap());
    s = RE_MASK.replace(&s, "").to_string();

    // 5. Trim leading/trailing whitespace and punctuation
    s.trim_matches(|c: char| c.is_whitespace() || c.is_ascii_punctuation()).to_string()
}

static OVERRIDES: Lazy<Vec<(Regex, &'static str, &'static str)>> = Lazy::new(|| {
    vec![
        (Regex::new(r"^TWINT\s+\*Sent\s+to\s+([LT](?:\.H[A-Z]*)?\.?|K\.H[A-Z]*\.?)(?:\s|$)").unwrap(), "Family", "Children"),
        (Regex::new(r"^TWINT\s+\*Sent\s+to\s+").unwrap(), "TWINT P2P", "Transfers"),
        (Regex::new(r"^TWINT\s+\*UBS\s+TWINT").unwrap(), "UBS TWINT", "Transfers"),
        (Regex::new(r"(?i)^(MIGROS|COOP|ALDI|LIDL|DENNER)\b").unwrap(), "Normalise", "Groceries"),
        (Regex::new(r"(?i)UBS Rest\.").unwrap(), "UBS Staff Restaurant", "Dining"),
    ]
});

static SECTOR_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // Dining
    for s in ["Restaurants", "Fast Food Restaurant", "Fast-Food Restaurants", "Caterers", "Bakeries"] {
        m.insert(s, "Dining");
    }
    
    // Groceries
    for s in ["Grocery stores", "Candy and nut stores", "Package stores - beer", "Freezer and locker meat provisioners"] {
        m.insert(s, "Groceries");
    }
    
    // Travel
    for s in ["Hotels", "Travel agencies", "Tourist Attractions and Exhibits", "Camp grounds", "Aparments", 
              "Airlines", "British Airways", "Swiss International Air Lines", "KLM", "Lufthansa", "LOT (Poland)", "Airports"] {
        m.insert(s, "Travel");
    }
    
    // Transport
    for s in ["Commuter transportation", "Passenger railways", "Bus lines", "Taxicabs", "Gasoline service stations", 
              "Parking & Garages", "Automobile services", "Toll and bridge fees", "Car Rental Company", "Fines"] {
        m.insert(s, "Transport");
    }
    
    // Healthcare
    for s in ["Pharmacies", "Doctors and Physicians", "Hospitals", "Dentists and Orthodontists", "Optician"] {
        m.insert(s, "Healthcare");
    }
    
    // Shopping
    for s in ["Clothing store", "Clothing - sports", "Shoe stores", "Department stores", "Furniture", 
              "Electronics Stores", "Home supply warehouse stores", "Hardware stores", "Cosmetic stores", 
              "Book stores", "Office supply stores", "Secondhand stores", "Household appliance stores", 
              "Garden and hardware center", "Games and hobby stores", "Leather goods", "Jewelry stores", 
              "Clock or jewelry or watch stores", "Non-durable Goods (B2B)", "Retail business",
              "Barber or beauty shops", "Cleaning - laundry and garment services"] {
        m.insert(s, "Shopping");
    }
    
    // Subscriptions
    for s in ["Digital goods", "Computer software stores", "Computer network/Information services", 
              "Continuity / Subscription Merchant", "Membership Organizations", "Misc. publishing and printing services", 
              "Books & newspapers (B2B)", "Telegraph services", "Data processing services", 
              "Films / Video production / distribution", "Bands Orchestras & Music Entertainment"] {
        m.insert(s, "Subscriptions");
    }
    
    // Fees
    for s in ["Government Services", "Postal Services", "Money orders - wire transfer"] {
        m.insert(s, "Fees");
    }
    
    // Children
    m.insert("Schools and Educational Services", "Children");
    
    // Other
    for s in ["Theather Production / Ticket Agencies", "Cinema", "Recreation Services", "Commercial Sports", "Banks - merchandise and services"] {
        m.insert(s, "Other");
    }

    // Housing
    for s in ["Electric utilities"] {
        m.insert(s, "Housing");
    }
    
    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalise_merchant() {
        assert_eq!(normalise_merchant("Phills BBQ              Cham         CHE"), "Phills BBQ Cham");
        assert_eq!(normalise_merchant("MERCHANT NAME XXXXXXXXXX 1234 CHE"), "MERCHANT NAME");
        assert_eq!(normalise_merchant("MIGROS ZH TIEFE           ZURICH       CHE"), "MIGROS ZH TIEFE ZURICH");
        assert_eq!(normalise_merchant("Coop-1234               Bern         CHE"), "Coop-1234 Bern");
        assert_eq!(normalise_merchant("APPLE.COM/BILL          CUPERTINO    USA"), "APPLE.COM/BILL CUPERTINO");
    }

    #[test]
    fn test_twint_family() {
        // Positive cases
        for s in ["L", "L.H", "T.H", "K.H", "L.HF"] {
            let desc = format!("TWINT *Sent to {} 076***1234 CHE", s);
            let res = classify(&desc, None).unwrap();
            assert_eq!(res.category, "Children", "Failed for {}", s);
            assert_eq!(res.merchant, "Family");
        }

        // Negative cases
        assert_eq!(classify("TWINT *Sent to T.P 079***9999 CHE", None).unwrap().category, "Transfers");
        assert_eq!(classify("TWINT *Sent to J.F 079***9999 CHE", None).unwrap().category, "Transfers");
        assert_eq!(classify("TWINT *Sent to K 079***9999 CHE", None).unwrap().category, "Transfers"); // K bare is Transfers
    }

    #[test]
    fn test_twint_other() {
        assert_eq!(classify("TWINT *UBS TWINT Zürich CHE", None).unwrap().category, "Transfers");
        assert_eq!(classify("TWINT *UBS TWINT", None).unwrap().merchant, "UBS TWINT");
    }

    #[test]
    fn test_overrides_precedence() {
        // Supermarket override beats Fast-Food sector
        let res = classify("Migros MR Brunaupark Zürich CHE", Some("Fast-Food Restaurants")).unwrap();
        assert_eq!(res.category, "Groceries");
        assert_eq!(res.merchant, "Migros MR Brunaupark Zürich");

        // UBS staff cafeteria override beats Hotels sector
        let res = classify("UBS Rest. Flur Zürich Zürich CHE", Some("Hotels")).unwrap();
        assert_eq!(res.category, "Dining");
        assert_eq!(res.merchant, "UBS Staff Restaurant");
    }

    #[test]
    fn test_sector_lookup_buckets() {
        let buckets = [
            ("Restaurants", "Dining"),
            ("Grocery stores", "Groceries"),
            ("Hotels", "Travel"),
            ("Airlines", "Travel"),
            ("Passenger railways", "Transport"),
            ("Pharmacies", "Healthcare"),
            ("Clothing store", "Shopping"),
            ("Digital goods", "Subscriptions"),
            ("Government Services", "Fees"),
            ("Schools and Educational Services", "Children"),
            ("Barber or beauty shops", "Shopping"),
            ("Cinema", "Other"),
        ];

        for (sector, expected_cat) in buckets {
            let res = classify("Some Merchant", Some(sector)).unwrap();
            assert_eq!(res.category, expected_cat, "Failed for sector {}", sector);
        }
    }

    #[test]
    fn test_empty_sector_no_override() {
        assert!(classify("Unknown Merchant", None).is_none());
        assert!(classify("Unknown Merchant", Some("   ")).is_none());
    }

    #[test]
    fn test_unknown_sector() {
        // Unknown sector should return None (fall through)
        assert!(classify("Unknown Merchant", Some("Future Sector")).is_none());
    }

    #[test]
    fn test_twint_family_real_data() {
        assert_eq!(
            classify("TWINT  *Sent to L.H.     076***0912   CHE", None).unwrap().category,
            "Children"
        );
        assert_eq!(
            classify("TWINT  *Sent to K.H.     076***0001   CHE", None).unwrap().category,
            "Children"
        );
        assert_eq!(
            classify("TWINT  *Sent to T.H.     076***0002   CHE", None).unwrap().category,
            "Children"
        );
        assert_eq!(
            classify("TWINT  *Sent to J.V.     079***8439   CHE", None).unwrap().category,
            "Transfers"
        );
        assert_eq!(
            classify("TWINT  *Sent to K.       079***3438   CHE", None).unwrap().category,
            "Transfers"
        );
    }
}
