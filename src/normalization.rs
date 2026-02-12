use std::collections::HashMap;

use super::shared_types::Market;

/// Normalizes market data, including timestamp alignment and string sanitization.
pub fn normalize_markets(markets: &mut Vec<Market>) {
    // Step 1.1: Timestamp Alignment
    // Group by neg_risk_market_id and force latest end_date
    let mut neg_risk_groups: HashMap<String, Vec<&mut Market>> = HashMap::new();
    for market in markets.iter_mut() {
        if let Some(ref neg_id) = market.neg_risk_market_id {
            neg_risk_groups
                .entry(neg_id.clone())
                .or_default()
                .push(market);
        }
    }

    for (_, group) in neg_risk_groups.iter_mut() {
        if let Some(latest_date) = group.iter().map(|m| m.end_date).max() {
            for market in group.iter_mut() {
                market.end_date = latest_date;
            }
        }
    }

    // Step 1.2: String Sanitization
    for market in markets {
        market.title = sanitize_string(&market.title);
        for condition in &mut market.conditions {
            condition.name = sanitize_string(&condition.name);
        }
    }
}

/// Helper function to sanitize strings: lowercase, remove stop words, standardize separators.
fn sanitize_string(s: &str) -> String {
    let s_lower = s.to_lowercase();

    // Remove punctuation
    let s_clean: String = s_lower
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect();

    let stop_words = vec![
        "the", "will", "be", "outcome", "a", "an", "is", "of", "in", "and",
    ];

    let words: Vec<&str> = s_clean
        .split_whitespace()
        .filter(|w| !stop_words.contains(w))
        .collect();

    words.join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_string() {
        assert_eq!(
            sanitize_string("Will Donald Trump win?"),
            "donald_trump_win"
        );
        assert_eq!(
            sanitize_string("The outcome of the election is..."),
            "election"
        );
        assert_eq!(
            sanitize_string("NBA: Lakers vs Warriors"),
            "nba_lakers_vs_warriors"
        );
    }
}
