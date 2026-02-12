use dotenv::dotenv;
use ethers::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let rpc_url_str = env::var("POLYGON_RPC_URL").expect("POLYGON_RPC_URL not set");
    let drpc_key = env::var("DRPC_API_KEY").ok();

    println!("Connecting to dRPC...");

    let mut headers = HeaderMap::new();
    if let Some(key) = drpc_key {
        headers.insert("Drpc-Key", HeaderValue::from_str(&key)?);
        println!("Using dRPC API Key.");
    }

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let url = Url::parse(&rpc_url_str)?;
    let provider = Provider::new(Http::new_with_client(url, http_client));

    let block_number = provider.get_block_number().await?;
    println!("✅ Connection Successful. Current Block: {}", block_number);

    if let Ok(private_key) = env::var("PRIVATE_KEY") {
        let wallet = private_key.parse::<LocalWallet>()?;
        let balance = provider.get_balance(wallet.address(), None).await?;
        println!("✅ Wallet: {:?}", wallet.address());
        println!("✅ Balance: {} POL", ethers::utils::format_ether(balance));
    }

    Ok(())
}
