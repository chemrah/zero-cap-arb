use serde::{Deserialize, Serialize};

// ─── Chain & DEX ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub id: u64,
    pub name: String,
    pub rpc_url: String,
    pub native_currency: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexConfig {
    pub name: String,
    pub address: String,
    pub chain_id: u64,
    pub router_abi: Option<String>,
}

// ─── Price Data ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub token: String,
    pub token_address: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub dex_name: String,
    pub price_usd: f64,
    pub liquidity_usd: f64,
    pub timestamp: u64,
}

/// All prices response – shows every DEX price across every chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllPricesResponse {
    pub token: String,
    pub token_address: String,
    pub prices: Vec<TokenPrice>,
    pub chain_summary: Vec<ChainSummary>,
    pub scan_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSummary {
    pub chain_id: u64,
    pub chain_name: String,
    pub dex_count: usize,
    pub min_price: f64,
    pub max_price: f64,
    pub avg_price: f64,
    pub spread_pct: f64,
}

// ─── Simple Arbitrage (Buy on X, sell on Y) ──────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub token_symbol: String,
    pub token_address: String,
    pub buy_chain_id: u64,
    pub buy_chain_name: String,
    pub buy_dex: String,
    pub buy_price_usd: f64,
    pub sell_chain_id: u64,
    pub sell_chain_name: String,
    pub sell_dex: String,
    pub sell_price_usd: f64,
    pub spread_pct: f64,
    pub estimated_profit_usd: f64,
    pub liquidity_usd: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarScanResponse {
    pub token: String,
    pub opportunities: Vec<ArbitrageOpportunity>,
    pub scan_time_ms: u64,
}

// ─── Advanced Arbitrage Strategies ────────────────────

/// Every arbitrage strategy the engine can execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitrageStrategy {
    /// Flash loan: borrow, buy cheap, sell expensive, repay
    FlashLoanArbitrage,
    /// Direct swap: user provides capital, find best route
    DirectSwap,
    /// Mint arbitrage: mint synthetic (e.g. DAI) cheaper than market
    MintArbitrage,
    /// Triangular: A → B → C → A on same chain
    TriangularArbitrage,
    /// Cross-chain: bridge asset across chains
    CrossChainArbitrage,
    /// JIT liquidity / MEV
    JitLiquidity,
}

impl ArbitrageStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArbitrageStrategy::FlashLoanArbitrage => "Flash Loan Arbitrage",
            ArbitrageStrategy::DirectSwap => "Direct Swap",
            ArbitrageStrategy::MintArbitrage => "Mint Arbitrage",
            ArbitrageStrategy::TriangularArbitrage => "Triangular Arbitrage",
            ArbitrageStrategy::CrossChainArbitrage => "Cross-Chain Arbitrage",
            ArbitrageStrategy::JitLiquidity => "JIT Liquidity / MEV",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ArbitrageStrategy::FlashLoanArbitrage => "Borrow via flash loan, buy on cheap DEX, sell on expensive DEX, repay loan + keep profit. 0 capital required.",
            ArbitrageStrategy::DirectSwap => "User provides the capital. We find the single best route via ParaSwap split routing across all DEXes.",
            ArbitrageStrategy::MintArbitrage => "Mint DAI on Spark/Maker when the mint cost is below market price. Sell DAI on DEX for profit.",
            ArbitrageStrategy::TriangularArbitrage => "Cycle through 3 tokens (e.g. DAI → ETH → USDC → DAI) on a single chain where price inconsistencies create profit.",
            ArbitrageStrategy::CrossChainArbitrage => "Bridge tokens across chains using Stargate/Across. Buy on chain A, bridge, sell on chain B where price is higher.",
            ArbitrageStrategy::JitLiquidity => "Provide just-in-time liquidity to an upcoming large swap. Capture the spread as the LP fee.",
        }
    }

    /// Whether this strategy needs 0 upfront capital
    pub fn zero_capital(&self) -> bool {
        match self {
            ArbitrageStrategy::FlashLoanArbitrage => true,
            ArbitrageStrategy::MintArbitrage => true,
            _ => false,
        }
    }

    pub fn needs_user_capital(&self) -> bool {
        !self.zero_capital()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularLeg {
    pub from_token: String,
    pub to_token: String,
    pub dex: String,
    pub rate: f64,
    pub expected_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularOpportunity {
    pub id: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub legs: Vec<TriangularLeg>,
    pub start_token: String,
    pub end_token: String,
    pub net_profit_pct: f64,
    pub estimated_profit_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainOpportunity {
    pub id: String,
    pub token: String,
    pub buy_chain_id: u64,
    pub buy_chain_name: String,
    pub buy_price: f64,
    pub sell_chain_id: u64,
    pub sell_chain_name: String,
    pub sell_price: f64,
    pub bridge_fee_usd: f64,
    pub net_profit_pct: f64,
    pub estimated_profit_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintOpportunity {
    pub id: String,
    pub token: String,
    pub mint_platform: String,
    pub mint_cost_usd: f64,
    pub market_price_usd: f64,
    pub spread_pct: f64,
    pub estimated_profit_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitLiquidityOpportunity {
    pub id: String,
    pub token: String,
    pub pool: String,
    pub dex: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub expected_fee_usd: f64,
    pub capital_required_usd: f64,
}

/// Simple execute request (backward compat)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteArbitrageRequest {
    pub opportunity_id: String,
    pub flash_loan_source: FlashLoanSource,
    pub gas_strategy: GasStrategy,
    pub user_address: String,
}

/// Unified view of all opportunities across all strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllOpportunitiesResponse {
    pub token: String,
    pub simple_arbitrages: Vec<ArbitrageOpportunity>,
    pub triangular_arbitrages: Vec<TriangularOpportunity>,
    pub cross_chain_arbitrages: Vec<CrossChainOpportunity>,
    pub mint_opportunities: Vec<MintOpportunity>,
    pub jit_opportunities: Vec<JitLiquidityOpportunity>,
    pub scan_time_ms: u64,
}

// ─── Execution ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlashLoanSource {
    AaveV3,
    RadiantV2,
    Spark,
}

impl FlashLoanSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlashLoanSource::AaveV3 => "Aave V3",
            FlashLoanSource::RadiantV2 => "Radiant V2",
            FlashLoanSource::Spark => "Spark Protocol",
        }
    }
    pub fn fee_pct(&self, token: &str) -> f64 {
        match self {
            FlashLoanSource::Spark => if token == "DAI" { 0.0 } else { 0.05 },
            FlashLoanSource::AaveV3 => 0.05,
            FlashLoanSource::RadiantV2 => 0.03,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GasStrategy {
    Flashbots,
    Pimlico,
    ZeroDev,
}

impl GasStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            GasStrategy::Flashbots => "Flashbots / MEV-Share",
            GasStrategy::Pimlico => "Pimlico (ERC-4337)",
            GasStrategy::ZeroDev => "ZeroDev (ERC-4337)",
        }
    }
}

/// Execution mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    FlashLoan,
    DirectSwap,
    Mint,
}

/// Master execution request for any strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedExecuteRequest {
    pub strategy: String, // "flash_loan" | "direct_swap" | "mint" | "triangular" | "cross_chain" | "jit"
    pub execution_mode: ExecutionMode,
    pub flash_loan_source: Option<FlashLoanSource>,
    pub gas_strategy: GasStrategy,
    pub user_address: String,
    pub token: Option<String>,
    pub token_address: Option<String>,
    pub amount: Option<String>,
    pub chain_id: Option<u64>,
    pub opportunity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub status: String,
    pub message: String,
    pub strategy: String,
    pub execution_mode: String,
    pub tx_hash: Option<String>,
    pub estimated_profit_usd: Option<f64>,
    pub gas_cost_usd: Option<f64>,
}

// ─── Velora (formerly ParaSwap) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeloraRoute {
    pub src_token: String,
    pub src_decimals: u8,
    pub dest_token: String,
    pub dest_decimals: u8,
    pub src_amount: String,
    pub dest_amount: String,
    pub percentage: f64,
    pub exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeloraPriceResponse {
    pub src_token: String,
    pub dest_token: String,
    pub src_amount: String,
    pub dest_amount: String,
    pub price_impact: f64,
    pub routes: Vec<VeloraRoute>,
    pub gas_cost_usd: f64,
    pub contract_address: String,
    pub token_transfer_proxy: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeloraTxResponse {
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: String,
    pub gas_price: String,
    pub gas: String,
    pub chain_id: u64,
}

// ─── Misc ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub msg_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub chains_connected: Vec<String>,
    pub uptime_secs: u64,
}
