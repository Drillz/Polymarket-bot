use polymarket_bot::arbitrage_engine::{check_combinatorial_pair, analyze_dependency};
use polymarket_bot::shared_types::{Market, Condition};
use rust_decimal_macros::dec;
use chrono::NaiveDate;
use std::time::Instant;

fn create_market(id: &str, title: &str, conditions: usize) -> Market {
    Market {
        id: id.to_string(),
        title: title.to_string(),
        end_date: NaiveDate::from_ymd_opt(2024, 11, 5).unwrap(),
        conditions: (0..conditions).map(|i| Condition {
            name: format!("Option {}", i),
            price: dec!(0.5),
            outcome: Some(true),
            asset_id: format!("{}_{}", id, i),
        }).collect(),
        neg_risk_market_id: None,
        tags: vec![],
    }
}

fn main() {
    let m1 = create_market("m1", "Will Donald Trump win the 2024 US Presidential Election?", 5);
    let m2 = create_market("m2", "Trump margin of victory in 2024 election > 5%", 5);

    let start = Instant::now();
    for _ in 0..10000 {
        check_combinatorial_pair(&m1, &m2);
    }
    let duration = start.elapsed();
    println!("Time taken: {:?}", duration);
}
