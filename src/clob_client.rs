use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use rust_decimal::Decimal;
use tokio::time::{sleep, Duration};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscriptionMessage {
    pub r#type: String,
    pub topic: String,
    pub asset_ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct PriceUpdate {
    pub asset_id: String,
    pub price: Decimal,
}

pub struct ClobClient {
    pub ws_url: String,
}

impl ClobClient {
    pub fn new() -> Self {
        let ws_url = env::var("CLOB_WS_URL").unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string());
        Self {
            ws_url,
        }
    }

    pub async fn stream_prices<F, Fut>(&self, asset_ids: Vec<String>, callback: F) -> Result<(), Box<dyn std::error::Error>> 
    where
        F: Fn(PriceUpdate) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let url = Url::parse(&self.ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();

        println!("Connected to CLOB WebSocket. Batching subscriptions...");

        for chunk in asset_ids.chunks(50) {
            let sub = serde_json::json!({
                "type": "subscribe",
                "topic": "prices",
                "asset_ids": chunk.to_vec(),
            });
            write.send(Message::Text(sub.to_string())).await?;
            sleep(Duration::from_millis(100)).await;
        }

        println!("All {} assets subscribed. Entering live stream.", asset_ids.len());

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(update) = serde_json::from_str::<PriceUpdate>(&text) {
                                callback(update).await;
                            }
                        }
                        Some(Ok(Message::Ping(payload))) => {
                            let _ = write.send(Message::Pong(payload)).await;
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            return Err("Connection closed by server".into());
                        }
                        Some(Err(e)) => return Err(Box::new(e)),
                        _ => (),
                    }
                }
                _ = sleep(Duration::from_secs(20)) => {
                    let _ = write.send(Message::Ping(vec![])).await;
                }
            }
        }
    }

    pub async fn place_order(&self, asset_id: &str, price: Decimal, size: Decimal, side: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("[CLOB] Placing {} order for {} at {} (Size: {})", side, asset_id, price, size);
        Ok(())
    }
}