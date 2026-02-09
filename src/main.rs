use dotenv::dotenv;
use polymarket_bot::arbitrage_engine::{
    are_markets_related, check_combinatorial_pair, check_rebalancing,
};
use polymarket_bot::blockchain::TradeExecutor;
use polymarket_bot::clob_client::ClobClient;
use polymarket_bot::market_fetcher::fetch_markets;
use polymarket_bot::normalization::normalize_markets;
use polymarket_bot::shared_types::DependencyGraph;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("Fetching markets from Polymarket...");
    let mut markets = fetch_markets().await?;
    println!("Fetched {} markets. Normalizing...", markets.len());
    normalize_markets(&mut markets);

    // Initialize Trader with dRPC support
    let executor =
        if let (Ok(rpc), Ok(key)) = (env::var("POLYGON_RPC_URL"), env::var("PRIVATE_KEY")) {
            println!("Wallet credentials found. Initializing Trade Executor...");
            let drpc_key = env::var("DRPC_API_KEY").ok();
            if drpc_key.is_some() {
                println!("dRPC API Key detected. Enabling MEV-protected HFT execution path.");
            }
            Some(Arc::new(TradeExecutor::new(&rpc, &key, drpc_key).await?))
        } else {
            println!("No wallet credentials found. Running in Scan-Only mode.");
            None
        };

    println!("Building Dependency Graph...");
    let mut dependency_graph = DependencyGraph::default();
    let mut market_id_to_idx = HashMap::new();
    for (i, m) in markets.iter().enumerate() {
        market_id_to_idx.insert(m.id.clone(), i);
    }

    let mut adjacency_list: HashMap<usize, Vec<usize>> = HashMap::new();
    let market_count = markets.len();
    for i in 0..market_count {
        for j in (i + 1)..market_count {
            if are_markets_related(&markets[i], &markets[j]) {
                dependency_graph
                    .related_markets
                    .push((markets[i].id.clone(), markets[j].id.clone()));
                adjacency_list.entry(i).or_default().push(j);
                adjacency_list.entry(j).or_default().push(i);
            }
        }
    }

    println!(
        "Found {} related market pairs.",
        dependency_graph.related_markets.len()
    );

    let mut asset_map = HashMap::new();
    let mut asset_ids = Vec::new();
    for (m_idx, market) in markets.iter().enumerate() {
        for (c_idx, condition) in market.conditions.iter().enumerate() {
            if !condition.asset_id.is_empty() {
                asset_map.insert(condition.asset_id.clone(), (m_idx, c_idx));
                asset_ids.push(condition.asset_id.clone());
            }
        }
    }

    let shared_markets = Arc::new(RwLock::new(markets));
    let shared_asset_map = Arc::new(asset_map);
    let shared_adjacency = Arc::new(adjacency_list);
    let shared_executor = executor;

    println!("--- ENTERING FERRARI MODE (WebSocket Streaming) ---");
    let clob_client = ClobClient::new();
    let mut reconnect_delay = 2;

    loop {
        let markets_lock = shared_markets.clone();
        let asset_map = shared_asset_map.clone();
        let adjacency = shared_adjacency.clone();
        let exec = shared_executor.clone();
        let ids = asset_ids.clone();

        let callback = move |update: polymarket_bot::clob_client::PriceUpdate| {
            let markets_lock = markets_lock.clone();
            let asset_map = asset_map.clone();
            let adjacency = adjacency.clone();
            let exec = exec.clone();

            async move {
                if let Some(&(m_idx, c_idx)) = asset_map.get(&update.asset_id) {
                    let mut markets = markets_lock.write().await;
                    markets[m_idx].conditions[c_idx].price = update.price;

                    if let Some(op) = check_rebalancing(&markets[m_idx]) {
                        println!(
                            "⚡ [HFT] Rebalancing Opp: {} Profit: {}",
                            op.market_id, op.profit
                        );
                        if let Some(e) = &exec {
                            let _ = e.execute_rebalancing(&op.market_id, dec!(100)).await;
                        }
                    }

                    if let Some(related_indices) = adjacency.get(&m_idx) {
                        for &r_idx in related_indices {
                            let ops = check_combinatorial_pair(&markets[m_idx], &markets[r_idx]);
                            for op in ops {
                                println!(
                                    "⚡ [HFT] Combinatorial Opp: {} <-> {} Profit: {}",
                                    op.market_id_1, op.market_id_2, op.profit
                                );
                                if let Some(e) = &exec {
                                    let _ = e
                                        .execute_combinatorial(
                                            &op.market_id_1,
                                            &op.market_id_2,
                                            dec!(100),
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
        };

        match clob_client.stream_prices(ids, callback).await {
            Ok(_) => {
                println!("WebSocket stream finished normally.");
                reconnect_delay = 2;
            }
            Err(e) => {
                eprintln!(
                    "WebSocket Error: {}. Reconnecting in {}s...",
                    e, reconnect_delay
                );
                sleep(Duration::from_secs(reconnect_delay)).await;
                reconnect_delay = std::cmp::min(reconnect_delay * 2, 60);
            }
        }
    }
}
