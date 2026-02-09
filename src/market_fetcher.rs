use crate::shared_types::{Condition, Market};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ApiEvent {
    #[serde(rename = "endDate")]
    end_date: Option<String>,
    #[serde(default)]
    markets: Vec<ApiMarket>,
    #[serde(default)]
    tags: Vec<ApiTag>,
}

#[derive(Deserialize, Debug)]
struct ApiTag {
    label: String,
}

#[derive(Deserialize, Debug)]
struct ApiMarket {
    id: String,
    question: String,
    #[serde(rename = "negRiskMarketID")]
    neg_risk_market_id: Option<String>,
    outcomes: Option<String>, // Often a JSON string like "["Yes", "No"]"
    #[serde(rename = "outcomePrices")]
    outcome_prices: Option<String>, // Often a JSON string like "["0.5", "0.5"]"
    #[serde(rename = "clobTokenIds")]
    clob_token_ids: Option<String>, // JSON string of token addresses
}

pub async fn fetch_markets() -> Result<Vec<Market>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    // Fetching open events with a limit to avoid too much data initially
    // Using a user-agent is often good practice
    let events: Vec<ApiEvent> = client
        .get("https://gamma-api.polymarket.com/events?closed=false&limit=50")
        .header("User-Agent", "PolymarketArbitrageBot/1.0")
        .send()
        .await?
        .json()
        .await?;

    let mut markets = Vec::new();

    for event in events {
        // Need a valid end_date
        let end_date = match event.end_date {
            Some(d) => match d.split('T').next().unwrap_or("").parse::<NaiveDate>() {
                Ok(date) => date,
                Err(_) => continue,
            },
            None => continue,
        };

        let tags: Vec<String> = event.tags.into_iter().map(|t| t.label).collect();

        for api_market in event.markets {
            let outcomes_str = api_market.outcomes.unwrap_or_else(|| "[]".to_string());
            let prices_str = api_market
                .outcome_prices
                .unwrap_or_else(|| "[]".to_string());
            let token_ids_str = api_market
                .clob_token_ids
                .unwrap_or_else(|| "[]".to_string());

            // Need to parse these JSON strings manually as they are often stringified JSON in the API
            let outcomes: Vec<String> = serde_json::from_str(&outcomes_str).unwrap_or_default();
            let prices: Vec<String> = serde_json::from_str(&prices_str).unwrap_or_default();
            let token_ids: Vec<String> = serde_json::from_str(&token_ids_str).unwrap_or_default();

            if outcomes.len() != prices.len() || outcomes.len() != token_ids.len() {
                continue;
            }

            let mut conditions = Vec::new();
            for (i, outcome_name) in outcomes.iter().enumerate() {
                let name_lower = outcome_name.to_lowercase();
                let outcome_bool = if name_lower == "yes" {
                    Some(true)
                } else if name_lower == "no" {
                    Some(false)
                } else {
                    None
                };

                if let Ok(price) = prices[i].parse::<Decimal>() {
                    conditions.push(Condition {
                        name: outcome_name.clone(),
                        price,
                        outcome: outcome_bool,
                        asset_id: token_ids[i].clone(),
                    });
                }
            }

            markets.push(Market {
                id: api_market.id,
                title: api_market.question, // Using question as title for the market
                end_date,
                conditions,
                neg_risk_market_id: api_market.neg_risk_market_id,
                tags: tags.clone(),
            });
        }
    }

    Ok(markets)
}
