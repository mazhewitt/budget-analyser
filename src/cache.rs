pub fn normalise_merchant_key(description: &str) -> String {
    let mut s = description.to_uppercase();

    // Strip known suffixes
    for suffix in &["EBILL-RECHT", "EBILL"] {
        if let Some(pos) = s.find(suffix) {
            s = s[..pos].to_string();
        }
    }

    // Split into tokens and process
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let mut result_tokens: Vec<&str> = Vec::new();

    for token in &tokens {
        // Skip "KARTE" and tokens that follow it (card number fragments)
        if *token == "KARTE" {
            break;
        }
        // Skip purely numeric tokens (reference numbers, transaction fragments)
        if token.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        result_tokens.push(token);
    }

    // Strip trailing all-digit tokens (already handled above, but also strip
    // from the end if mixed tokens remain followed by digits)
    while let Some(last) = result_tokens.last() {
        if last.chars().all(|c| c.is_ascii_digit()) {
            result_tokens.pop();
        } else {
            break;
        }
    }

    result_tokens.join(" ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbb_mobile_grouping() {
        let a = normalise_merchant_key("SBB MOBILE");
        let b = normalise_merchant_key("SBB MOBILE");
        assert_eq!(a, b);
        assert_eq!(a, "SBB MOBILE");
    }

    #[test]
    fn test_strip_ebill_suffix() {
        let key = normalise_merchant_key("Steuerverwaltung EBILL-RECHT");
        assert_eq!(key, "STEUERVERWALTUNG");
    }

    #[test]
    fn test_coop_variants_stay_separate() {
        let a = normalise_merchant_key("COOP PRONTO BHFSTR ZURICH");
        let b = normalise_merchant_key("COOP CITY BASEL");
        assert_ne!(a, b);
    }

    #[test]
    fn test_migros_reference_number_stripping() {
        let a = normalise_merchant_key("MIGROS BASEL M 01234 KARTE 1234");
        let b = normalise_merchant_key("MIGROS BASEL M 56789 KARTE 1234");
        assert_eq!(a, b);
        assert_eq!(a, "MIGROS BASEL M");
    }

    #[test]
    fn test_vivao_sympa_ebill() {
        let key = normalise_merchant_key("Vivao Sympa EBILL-RECHT");
        assert_eq!(key, "VIVAO SYMPA");
    }

    #[test]
    fn test_sunrise_ebill() {
        let key = normalise_merchant_key("Sunrise GmbH EBILL");
        assert_eq!(key, "SUNRISE GMBH");
    }
}
