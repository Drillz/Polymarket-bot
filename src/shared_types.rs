use rust_decimal::Decimal;
use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Market {
    pub id: String,
    pub title: String,
    pub end_date: NaiveDate,
    pub conditions: Vec<Condition>,
    pub neg_risk_market_id: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub name: String,
    pub price: Decimal,
    pub outcome: Option<bool>, // true for YES, false for NO
    pub asset_id: String,      // The token address/ID for this outcome
}

#[derive(Debug)]
pub struct RebalancingOpportunity {
    pub market_id: String,
    pub profit: Decimal,
    pub opportunity_type: String, // "Long" or "Short"
}

#[derive(Debug)]
pub struct CombinatorialOpportunity {
    pub market_id_1: String,
    pub market_id_2: String,
    pub condition_name_1: String,
    pub condition_name_2: String,
    pub profit: Decimal,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    C1ImpliesC2,
    C2ImpliesC1,
}

#[derive(Debug, Default)]
pub struct DependencyGraph {
    pub related_markets: Vec<(String, String)>, // Pairs of market IDs
    pub implications: HashMap<(String, String), Direction>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Entity {
    Candidate(String),
    Location(String),
    Event(String),
    NumericalValue(Decimal),
}

#[derive(Debug, Clone)]
pub enum PatternType {
    WinnerMargin,
    SubsetImplication,
    NumericRange,
}

#[derive(Debug)]
pub struct Dependency {
    pub pattern: PatternType,
    pub direction: Direction,
}