use super::shared_types::{Market, Condition, RebalancingOpportunity, CombinatorialOpportunity, Direction, DependencyGraph, Entity, PatternType, Dependency};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{HashMap, HashSet};
use regex::Regex;
use strsim::normalized_damerau_levenshtein;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_RANGE: Regex = Regex::new(r"(\d+\.?\d*)\s*-\s*(\d+\.?\d*)%?").unwrap();
    static ref RE_GREATER_THAN: Regex = Regex::new(r">(\d+\.?\d*)%?").unwrap();
    static ref RE_LESS_THAN: Regex = Regex::new(r"<(\d+\.?\d*)%?").unwrap();
}

/// Trait for different dependency patterns as per the design summary
trait DependencyPattern {
    fn matches(&self, t1: &str, c1: &Condition, t2: &str, c2: &Condition, shared_entities: &HashSet<Entity>) -> Option<Dependency>;
}

struct WinnerMarginPattern;
impl DependencyPattern for WinnerMarginPattern {
    fn matches(&self, t1: &str, c1: &Condition, t2: &str, c2: &Condition, shared_entities: &HashSet<Entity>) -> Option<Dependency> {
        // Use provided lowercased titles
        let is_winner_m = t1.contains("win") || t1.contains("winner") || t1.contains("victory");
        let is_margin_m = t2.contains("margin") || t2.contains("points") || t2.contains("by");

        if (is_winner_m && is_margin_m) || (is_margin_m && is_winner_m) {
            for entity in shared_entities {
                if let Entity::Candidate(name) = entity {
                    let name_lower = name.to_lowercase();
                    // Assuming c1.name is already normalized/lowercase from data ingestion
                    let m1_rel = t1.contains(&name_lower) || c1.name.contains(&name_lower);
                    let m2_rel = t2.contains(&name_lower) || c2.name.contains(&name_lower);
                    
                    if m1_rel && m2_rel && c1.outcome == Some(true) && c2.outcome == Some(true) {
                        if is_winner_m && is_margin_m {
                             return Some(Dependency { pattern: PatternType::WinnerMargin, direction: Direction::C2ImpliesC1 });
                        } else {
                             return Some(Dependency { pattern: PatternType::WinnerMargin, direction: Direction::C1ImpliesC2 });
                        }
                    }
                }
            }
        }
        None
    }
}

struct SubsetImplicationPattern;
impl DependencyPattern for SubsetImplicationPattern {
    fn matches(&self, t1: &str, c1: &Condition, t2: &str, c2: &Condition, _shared: &HashSet<Entity>) -> Option<Dependency> {
        if t2.contains(t1) && t1 != t2 {
            if c1.outcome == c2.outcome && c1.outcome == Some(true) {
                return Some(Dependency { pattern: PatternType::SubsetImplication, direction: Direction::C2ImpliesC1 });
            }
        } else if t1.contains(t2) && t1 != t2 {
            if c1.outcome == c2.outcome && c1.outcome == Some(true) {
                return Some(Dependency { pattern: PatternType::SubsetImplication, direction: Direction::C1ImpliesC2 });
            }
        }
        None
    }
}

struct StateNationalPattern;
impl DependencyPattern for StateNationalPattern {
    fn matches(&self, t1: &str, _c1: &Condition, t2: &str, _c2: &Condition, _shared: &HashSet<Entity>) -> Option<Dependency> {
        let is_state = |t: &str| t.contains("win") && (t.contains("pennsylvania") || t.contains("georgia") || t.contains("arizona"));
        let is_national = |t: &str| t.contains("win") && (t.contains("election") || t.contains("presidency"));
        
        if is_state(t1) && is_national(t2) {
             return Some(Dependency { pattern: PatternType::SubsetImplication, direction: Direction::C1ImpliesC2 });
        }
        None
    }
}

struct BalanceOfPowerPattern;
impl DependencyPattern for BalanceOfPowerPattern {
    fn matches(&self, t1: &str, _c1: &Condition, t2: &str, _c2: &Condition, _shared: &HashSet<Entity>) -> Option<Dependency> {
        let is_pres = t1.contains("presidency") || t1.contains("white house");
        let is_senate = t2.contains("senate");
        
        if is_pres && is_senate {
             return Some(Dependency { pattern: PatternType::SubsetImplication, direction: Direction::C1ImpliesC2 });
        }
        None
    }
}

struct NumericRangePattern;
impl DependencyPattern for NumericRangePattern {
    fn matches(&self, _t1: &str, c1: &Condition, _t2: &str, c2: &Condition, _shared: &HashSet<Entity>) -> Option<Dependency> {
        let r1 = parse_range(&c1.name)?;
        let r2 = parse_range(&c2.name)?;

        if r1.0 >= r2.0 && r1.1 <= r2.1 && (r1.0 > r2.0 || r1.1 < r2.1) {
            return Some(Dependency { pattern: PatternType::NumericRange, direction: Direction::C1ImpliesC2 });
        } else if r2.0 >= r1.0 && r2.1 <= r1.1 && (r2.0 > r1.0 || r2.1 < r1.1) {
            return Some(Dependency { pattern: PatternType::NumericRange, direction: Direction::C2ImpliesC1 });
        }
        None
    }
}

fn parse_range(name: &str) -> Option<(Decimal, Decimal)> {
    if let Some(caps) = RE_RANGE.captures(name) {
        let start = caps.get(1).unwrap().as_str().parse::<Decimal>().ok()?;
        let end = caps.get(2).unwrap().as_str().parse::<Decimal>().ok()?;
        return Some((start, end));
    }
    if let Some(caps) = RE_GREATER_THAN.captures(name) {
        let val = caps.get(1).unwrap().as_str().parse::<Decimal>().ok()?;
        return Some((val, Decimal::from(1_000_000)));
    }
    if let Some(caps) = RE_LESS_THAN.captures(name) {
        let val = caps.get(1).unwrap().as_str().parse::<Decimal>().ok()?;
        return Some((Decimal::from(0), val));
    }
    None
}

/// Helper function to encapsulate dependency logic with pre-computed values
fn analyze_dependency_inner(t1: &str, c1: &Condition, t2: &str, c2: &Condition, shared: &HashSet<Entity>) -> Option<Dependency> {
    let patterns: Vec<Box<dyn DependencyPattern>> = vec![
        Box::new(WinnerMarginPattern),
        Box::new(SubsetImplicationPattern),
        Box::new(NumericRangePattern),
        Box::new(StateNationalPattern),
        Box::new(BalanceOfPowerPattern),
    ];

    for pattern in patterns {
        if let Some(dep) = pattern.matches(t1, c1, t2, c2, shared) {
            return Some(dep);
        }
    }
    None
}

pub fn analyze_dependency(m1: &Market, c1: &Condition, m2: &Market, c2: &Condition) -> Option<Dependency> {
    if m1.id == m2.id { return None; }

    // Perform extraction and lowercasing here for singular calls (less optimized path)
    let t1 = m1.title.to_lowercase();
    let t2 = m2.title.to_lowercase();

    let entities1 = extract_entities(&t1);
    let entities2 = extract_entities(&t2);
    let shared: HashSet<_> = entities1.intersection(&entities2).cloned().collect();

    if shared.is_empty() && !t1.contains(&t2) && !t2.contains(&t1) {
        return None;
    }

    analyze_dependency_inner(&t1, c1, &t2, c2, &shared)
}

fn extract_entities(title: &str) -> HashSet<Entity> {
    let mut entities = HashSet::new();
    let keywords = [
        "trump", "biden", "harris", "walz", "vance", 
        "bitcoin", "eth", "solana", "fed", "inflation",
        "lakers", "warriors", "celtics", "knicks",
        "iran", "israel", "ukraine", "russia"
    ];
    
    // Split by underscores (as normalized) and remove punctuation
    for word in title.split('_') {
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if keywords.contains(&clean_word) {
            entities.insert(Entity::Candidate(clean_word.to_string()));
        }
    }
    entities
}

pub fn find_combinatorial_opportunities(
    markets: &[Market],
    dependency_graph: &DependencyGraph,
) -> Vec<CombinatorialOpportunity> {
    let mut opportunities = Vec::new();
    let market_map: HashMap<String, &Market> = markets.iter().map(|m| (m.id.clone(), m)).collect();

    for (market_id_1, market_id_2) in &dependency_graph.related_markets {
        if let (Some(m1), Some(m2)) = (market_map.get(market_id_1), market_map.get(market_id_2)) {
            opportunities.extend(check_combinatorial_pair(m1, m2));
        }
    }
    opportunities
}

/// Efficiently checks just two markets for combinatorial arbitrage
pub fn check_combinatorial_pair(m1: &Market, m2: &Market) -> Vec<CombinatorialOpportunity> {
    // Optimization: Hoist entity extraction and string lowercasing out of the loop
    let t1 = m1.title.to_lowercase();
    let t2 = m2.title.to_lowercase();

    let entities1 = extract_entities(&t1);
    let entities2 = extract_entities(&t2);
    let shared: HashSet<_> = entities1.intersection(&entities2).cloned().collect();

    // Early return if no relationship possible
    if shared.is_empty() && !t1.contains(&t2) && !t2.contains(&t1) {
        return Vec::new();
    }

    let mut opportunities = Vec::new();
    for c1 in &m1.conditions {
        for c2 in &m2.conditions {
            // Use optimized inner function
            if let Some(dep) = analyze_dependency_inner(&t1, c1, &t2, c2, &shared) {
                let (implying_c, implied_c) = match dep.direction {
                    Direction::C1ImpliesC2 => (c1, c2),
                    Direction::C2ImpliesC1 => (c2, c1),
                };

                if implying_c.price > implied_c.price {
                    opportunities.push(CombinatorialOpportunity {
                        market_id_1: m1.id.clone(),
                        market_id_2: m2.id.clone(),
                        condition_name_1: implying_c.name.clone(),
                        condition_name_2: implied_c.name.clone(),
                        profit: implying_c.price - implied_c.price,
                    });
                }
            }
        }
    }
    opportunities
}

pub fn check_rebalancing(market: &Market) -> Option<RebalancingOpportunity> {
    let sum_prices: Decimal = market.conditions.iter().map(|c| c.price).sum();
    let fee_threshold = dec!(0.02);

    if sum_prices < (dec!(1) - fee_threshold) {
        Some(RebalancingOpportunity {
            market_id: market.id.clone(),
            profit: dec!(1) - sum_prices,
            opportunity_type: "Long".to_string(),
        })
    } else if sum_prices > (dec!(1) + fee_threshold) {
        Some(RebalancingOpportunity {
            market_id: market.id.clone(),
            profit: sum_prices - dec!(1),
            opportunity_type: "Short".to_string(),
        })
    } else {
        None
    }
}

pub fn are_markets_related(m1: &Market, m2: &Market) -> bool {
    if m1.id == m2.id || m1.end_date != m2.end_date { return false; }
    let tags1: HashSet<_> = m1.tags.iter().collect();
    let tags2: HashSet<_> = m2.tags.iter().collect();
    if tags1.is_disjoint(&tags2) { return false; }
    normalized_damerau_levenshtein(&m1.title, &m2.title) > 0.6
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_types::{Market, Condition};
    use rust_decimal_macros::dec;
    use chrono::NaiveDate;

    #[test]
    fn test_rebalancing_detection() {
        let market = Market {
            id: "test".to_string(),
            title: "Test Market".to_string(),
            end_date: NaiveDate::from_ymd_opt(2024, 11, 5).unwrap(),
            conditions: vec![
                Condition { name: "Yes".to_string(), price: dec!(0.4), outcome: Some(true), asset_id: "1".to_string() },
                Condition { name: "No".to_string(), price: dec!(0.4), outcome: Some(false), asset_id: "2".to_string() },
            ],
            neg_risk_market_id: None,
            tags: vec![],
        };
        
        let opp = check_rebalancing(&market).unwrap();
        assert_eq!(opp.profit, dec!(0.2));
        assert_eq!(opp.opportunity_type, "Long");
    }

    #[test]
    fn test_numeric_range_implication() {
        let m1 = Market {
            id: "m1".to_string(),
            title: "trump_margin".to_string(),
            end_date: NaiveDate::from_ymd_opt(2024, 11, 5).unwrap(),
            conditions: vec![Condition { name: "5-10%".to_string(), price: dec!(0.6), outcome: Some(true), asset_id: "1".to_string() }],
            neg_risk_market_id: None,
            tags: vec![],
        };
        let m2 = Market {
            id: "m2".to_string(),
            title: "trump_margin".to_string(),
            end_date: NaiveDate::from_ymd_opt(2024, 11, 5).unwrap(),
            conditions: vec![Condition { name: "0-20%".to_string(), price: dec!(0.5), outcome: Some(true), asset_id: "2".to_string() }],
            neg_risk_market_id: None,
            tags: vec![],
        };
        
        let dep = analyze_dependency(&m1, &m1.conditions[0], &m2, &m2.conditions[0]).unwrap();
        assert_eq!(dep.direction, Direction::C1ImpliesC2);
    }

        #[test]

        fn test_winner_margin_implication() {

            let m1 = Market {

                id: "m1".to_string(),

                title: "trump_win_presidential_election".to_string(),

                end_date: NaiveDate::from_ymd_opt(2024, 11, 5).unwrap(),

                conditions: vec![Condition { name: "Donald Trump".to_string(), price: dec!(0.5), outcome: Some(true), asset_id: "1".to_string() }],

                neg_risk_market_id: None,

                tags: vec![],

            };

            let m2 = Market {

                id: "m2".to_string(),

                title: "trump_margin_victory".to_string(),

                end_date: NaiveDate::from_ymd_opt(2024, 11, 5).unwrap(),

                conditions: vec![Condition { name: "5-10%".to_string(), price: dec!(0.6), outcome: Some(true), asset_id: "2".to_string() }],

                neg_risk_market_id: None,

                tags: vec![],

            };

            

            let dep = analyze_dependency(&m1, &m1.conditions[0], &m2, &m2.conditions[0]).unwrap();

            assert_eq!(dep.direction, Direction::C2ImpliesC1);

        }

    }
