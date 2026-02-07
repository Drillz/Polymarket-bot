use crate::shared_types::Market;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarketCategory {
    Politics,
    Crypto,
    Sports,
    Economics,
    Science,
    Other,
}

pub struct TopicClassifier;

impl TopicClassifier {
    pub fn classify(market: &Market) -> MarketCategory {
        // 1. Check Tags first
        for tag in &market.tags {
            let t = tag.to_lowercase();
            if t.contains("politics") || t.contains("election") || t.contains("white house") {
                return MarketCategory::Politics;
            }
            if t.contains("crypto") || t.contains("bitcoin") || t.contains("ethereum") || t.contains("nft") {
                return MarketCategory::Crypto;
            }
            if t.contains("sport") || t.contains("nba") || t.contains("nfl") || t.contains("soccer") {
                return MarketCategory::Sports;
            }
            if t.contains("economy") || t.contains("fed") || t.contains("rates") || t.contains("inflation") {
                return MarketCategory::Economics;
            }
            if t.contains("science") || t.contains("space") || t.contains("covid") || t.contains("climate") {
                return MarketCategory::Science;
            }
        }

        // 2. Fallback to Title keywords
        let title_lower = market.title.to_lowercase();
        if title_lower.contains("trump") || title_lower.contains("biden") || title_lower.contains("senate") {
            return MarketCategory::Politics;
        }
        if title_lower.contains("btc") || title_lower.contains("eth") || title_lower.contains("sol") {
            return MarketCategory::Crypto;
        }
        if title_lower.contains("game") || title_lower.contains("match") || title_lower.contains("league") {
            return MarketCategory::Sports;
        }

        MarketCategory::Other
    }
}
