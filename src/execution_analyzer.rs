use rust_decimal::Decimal;
use std::collections::HashMap;

// Stub for User Execution Data
#[derive(Debug, Clone)]
pub struct UserExecution {
    pub user_address: String,
    pub market_id: String,
    pub outcome_index: usize,
    pub amount: Decimal,
    pub timestamp: u64,
}

pub struct ExecutionAnalyzer;

impl ExecutionAnalyzer {
    /// Groups executions by user and detects potential arbitrage activity
    pub fn analyze_executions(executions: &[UserExecution]) -> Vec<String> {
        let mut arbitrageurs = Vec::new();
        let mut user_activity: HashMap<String, Vec<&UserExecution>> = HashMap::new();

        // Group by user
        for exec in executions {
            user_activity.entry(exec.user_address.clone()).or_insert_with(Vec::new).push(exec);
        }

        // Analyze each user's patterns
        for (user, txs) in user_activity {
            if Self::is_arbitrage_pattern(txs) {
                arbitrageurs.push(user);
            }
        }

        arbitrageurs
    }

    /// Heuristic to detect if a set of transactions looks like arbitrage
    /// e.g., Buying YES and NO in the same market within a short window
    fn is_arbitrage_pattern(txs: Vec<&UserExecution>) -> bool {
        if txs.len() < 2 { return false; }

        let mut market_counts: HashMap<String, usize> = HashMap::new();
        for tx in &txs {
            *market_counts.entry(tx.market_id.clone()).or_default() += 1;
        }

        // If a user interacts with the same market multiple times (e.g. buying multiple outcomes)
        for (_, count) in market_counts {
            if count > 1 {
                // Further check: distinct outcomes? Short time window?
                // For this stub, simple count > 1 suggests multi-leg execution
                return true;
            }
        }
        
        false
    }
}
