use ethers::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::str::FromStr;
use std::env;
use reqwest::header::{HeaderMap, HeaderValue};
use url::Url;

// Polymarket CTF Exchange (Proxy) Address (Default: Mainnet)
const DEFAULT_CTF_EXCHANGE_ADDRESS: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";

abigen!(
    CtfExchange,
    r#"[
        event OrderFilled(bytes32 indexed orderHash, address indexed maker, address indexed taker, uint256 makerFillAmount, uint256 takerFillAmount, uint256 fee)
        function splitPosition(bytes32 conditionId, bytes32 parentCollectionId, bytes32 collectionId, uint256[] partition, uint256 amount) external
        function mergePositions(bytes32 conditionId, bytes32 parentCollectionId, bytes32 collectionId, uint256[] partition, uint256 amount) external
    ]"#
);

// Type alias for our middleware stack (Provider + Wallet)
type Client = SignerMiddleware<Provider<Http>, LocalWallet>;

pub struct TradeExecutor {
    #[allow(dead_code)]
    client: Arc<Client>,
    #[allow(dead_code)]
    contract: CtfExchange<Client>,
}

impl TradeExecutor {
    pub async fn new(rpc_url: &str, private_key: &str, drpc_key: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let url = Url::from_str(rpc_url)?;
        
        let mut headers = HeaderMap::new();
        if let Some(key) = drpc_key {
            // dRPC uses Drpc-Key header for authentication
            headers.insert("Drpc-Key", HeaderValue::from_str(&key)?);
        }

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let http_provider = Http::new_with_client(url, http_client);
        let provider = Provider::new(http_provider);
        let chain_id: U256 = provider.get_chainid().await?;
        
        let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());
        let client = Arc::new(SignerMiddleware::new(provider, wallet));
        
        let address_str = env::var("CTF_EXCHANGE_ADDRESS").unwrap_or_else(|_| DEFAULT_CTF_EXCHANGE_ADDRESS.to_string());
        let address = Address::from_str(&address_str)?;
        let contract = CtfExchange::new(address, client.clone());

        Ok(Self { client, contract })
    }

    pub async fn execute_rebalancing(&self, condition_id: &str, amount: Decimal) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        println!("🚀 [EXECUTION] Rebalancing Condition: {} Amount: {}", condition_id, amount);
        // This would call splitPosition or mergePositions based on the rebalancing type
        // For now, we simulate success until the specific contract interaction is finalized
        Ok(TransactionReceipt::default())
    }

    pub async fn execute_combinatorial(&self, market_1: &str, market_2: &str, amount: Decimal) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        println!("🚀 [EXECUTION] Combinatorial Trade: {} -> {} Amount: {}", market_1, market_2, amount);
        // This would execute the two legs of the trade on the CTF Exchange
        Ok(TransactionReceipt::default())
    }
}

pub struct BlockchainCollector {
    contract: CtfExchange<Provider<Http>>,
}

impl BlockchainCollector {
    pub fn new(rpc_url: &str, drpc_key: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let url = Url::from_str(rpc_url)?;
        
        let mut headers = HeaderMap::new();
        if let Some(key) = drpc_key {
            headers.insert("Drpc-Key", HeaderValue::from_str(&key)?);
        }

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let http_provider = Http::new_with_client(url, http_client);
        let provider = Provider::new(http_provider);
        let client = Arc::new(provider);
        let address_str = env::var("CTF_EXCHANGE_ADDRESS").unwrap_or_else(|_| DEFAULT_CTF_EXCHANGE_ADDRESS.to_string());
        let address = Address::from_str(&address_str)?;
        let contract = CtfExchange::new(address, client.clone());

        Ok(Self { contract })
    }

    pub async fn fetch_bids_batched(&self, from_block: u64, to_block: u64) -> Result<Vec<OrderFilledFilter>, Box<dyn std::error::Error>> {
        let filter = self.contract.order_filled_filter().from_block(from_block).to_block(to_block);
        let logs = filter.query().await?;
        Ok(logs)
    }
}

pub struct VwapCalculator;

impl VwapCalculator {
    pub fn calculate_vwap(fills: &[OrderFilledFilter]) -> Decimal {
        let mut total_vol = Decimal::ZERO;
        let mut total_cost = Decimal::ZERO;

        for fill in fills {
            let maker_amt = Decimal::from_str(&fill.maker_fill_amount.to_string()).unwrap_or_default();
            let taker_amt = Decimal::from_str(&fill.taker_fill_amount.to_string()).unwrap_or_default();

            if maker_amt.is_zero() { continue; }
            total_vol += maker_amt;
            total_cost += taker_amt;
        }

        if total_vol.is_zero() {
            Decimal::ZERO
        } else {
            total_cost / total_vol
        }
    }
}