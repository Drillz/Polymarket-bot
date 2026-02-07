# Polymarket HFT Arbitrage Bot

A high-frequency trading (HFT) bot written in **Rust** designed to detect and execute arbitrage opportunities on [Polymarket](https://polymarket.com/). This system uses **deterministic heuristics** and a local **dependency graph** to identify price discrepancies without relying on slow LLM calls, making it suitable for real-time execution.

## 🚀 Features

*   **⚡ High-Frequency Execution:** Built with `tokio` for asynchronous runtime and WebSocket streaming for real-time price updates.
*   **🔄 Rebalancing Arbitrage:** Automatically detects when the sum of outcome prices in a single market deviates significantly from $1.00 (risk-free profit).
*   **🔗 Combinatorial Arbitrage:** Identifies "Subset vs. Superset" mispricings between related markets (e.g., *Trump wins* vs. *Trump wins by >5%*).
*   **🧠 Deterministic Dependency Engine:** Uses Regex, Jaccard Similarity, and Subset Logic to build a market dependency graph offline—no external AI/LLM APIs required.
*   **🛡️ MEV Protection:** Integrated support for private RPC endpoints (e.g., dRPC) to minimize front-running risks.
*   **📡 WebSocket Streaming:** Subscribes to Polymarket's CLOB (Central Limit Order Book) via WebSocket for millisecond-latency updates.

## 🛠️ Architecture

The bot operates in several distinct phases:

1.  **Ingestion & Normalization:** Fetches all active markets and normalizes their data (standardizing dates, sanitizing strings).
2.  **Graph Construction:** Builds a `DependencyGraph` by clustering markets based on tags, end dates, and text similarity.
3.  **Real-Time Loop:**
    *   Connects to Polymarket's WebSocket.
    *   On every price update (`tick`), instantly checks for:
        *   **Rebalancing:** `Sum(Prices) < 0.98` or `Sum(Prices) > 1.02`.
        *   **Combinatorial:** `Price(Subset) > Price(Superset)`.
    *   **Execution:** Triggers a trade via the `TradeExecutor` if a profitable opportunity is found.

## 📋 Prerequisites

*   **Rust**: Stable channel (install via [rustup](https://rustup.rs/)).
*   **Git**: Version control.
*   **Polygon RPC**: A fast RPC URL for the Polygon PoS chain (e.g., Alchemy, Infura, or dRPC).
*   **Polymarket Account**: Private key and API credentials (if executing trades).

## ⚙️ Installation

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/Drillz/Polymarket-bot.git
    cd Polymarket-bot
    ```

2.  **Configure Environment**
    Create a `.env` file in the root directory:
    ```bash
    touch .env
    ```
    Add the following variables:
    ```env
    # Blockchain & Wallet
    POLYGON_RPC_URL=your_polygon_rpc_url
    PRIVATE_KEY=your_wallet_private_key
    
    # Optional: MEV Protection
    DRPC_API_KEY=your_drpc_key
    
    # Polymarket API (Optional/Future Use)
    POLY_API_KEY=your_poly_api_key
    POLY_API_SECRET=your_poly_api_secret
    POLY_PASSPHRASE=your_poly_passphrase
    ```

3.  **Build the Project**
    ```bash
    cargo build --release
    ```

## 🏃 Usage

### Scan-Only Mode (No Wallet)
If you don't provide a `PRIVATE_KEY` in the `.env` file, the bot will run in **Scan-Only Mode**. It will print opportunities to the console but will not attempt to execute trades.

```bash
cargo run --release
```

### Live Trading Mode
**⚠️ WARNING: Real funds will be used.**
Ensure your `.env` is fully configured and your wallet has MATIC for gas and USDC (bridged to Polygon) for trading.

```bash
cargo run --release
```

## 🧪 Testing

Run the unit tests to verify the arbitrage logic and dependency detection:

```bash
cargo test
```

## 📂 Project Structure

*   `src/main.rs`: Entry point. Orchestrates the WebSocket loop and initialization.
*   `src/arbitrage_engine.rs`: Core logic for `check_rebalancing` and `find_combinatorial_opportunities`.
*   `src/dependency_graph.rs`: Logic for building the map of related markets.
*   `src/clob_client.rs`: WebSocket client for streaming prices.
*   `src/normalization.rs`: Utilities for cleaning and standardizing market data.
*   `src/blockchain.rs`: Handles transaction signing and interaction with the Polygon network.

## ⚠️ Disclaimer

This software is for educational and research purposes only. **Use at your own risk.** High-frequency trading involves significant financial risk. The authors are not responsible for any financial losses incurred while using this bot. Always test thoroughly in a safe environment before deploying real capital.

## 📄 License

[MIT License](LICENSE)
