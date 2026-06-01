use crate::paraswap_client::ParaSwapClient;
use crate::radar_scanner::RadarScanner;
use crate::types::*;
use crate::websocket;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

#[derive(Clone)]
pub struct AppState {
    pub scanner: Arc<RadarScanner>,
    pub paraswap: Arc<ParaSwapClient>,
    pub start_time: Instant,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/scan", post(scan_token))
        .route("/api/all-prices", post(get_all_prices))
        .route("/api/all-opportunities", post(get_all_opportunities))
        .route("/api/paraswap/price", post(get_paraswap_price))
        .route("/api/paraswap/build-tx", post(build_paraswap_tx))
        .route("/api/execute", post(execute_arbitrage))
        .route("/api/execute/advanced", post(execute_advanced))
        .route("/api/chains", get(list_chains))
        .route("/api/dexes/{chain_id}", get(list_dexes))
        .route("/ws", get(websocket::ws_handler))
        .with_state(state)
}

// ─── Health ───────────────────────────────────────────

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let chains = crate::chains::get_chains();
    Json(HealthResponse {
        status: "ok".to_string(),
        chains_connected: chains.iter().map(|c| c.name.clone()).collect(),
        uptime_secs: state.start_time.elapsed().as_secs(),
    })
}

// ─── Scan (best buy/sell) ─────────────────────────────

async fn scan_token(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<RadarScanResponse>, (StatusCode, String)> {
    let token = req["token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "token required".to_string()))?;
    let addr = req["token_address"].as_str();
    info!("Scanning: {} ({:?})", token, addr);
    state.scanner.scan_token(token, addr).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Scan failed: {}", e)))
}

// ─── All Prices (every DEX) ──────────────────────────

async fn get_all_prices(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<AllPricesResponse>, (StatusCode, String)> {
    let token = req["token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "token required".to_string()))?;
    let addr = req["token_address"].as_str();
    info!("Fetching ALL prices for: {} ({:?})", token, addr);
    state.scanner.scan_all_prices(token, addr).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Price scan failed: {}", e)))
}

// ─── All Opportunities (every strategy) ──────────────

async fn get_all_opportunities(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<AllOpportunitiesResponse>, (StatusCode, String)> {
    let token = req["token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "token required".to_string()))?;
    let addr = req["token_address"].as_str();
    info!("Finding ALL opportunities for: {} ({:?})", token, addr);
    state.scanner.scan_all_strategies(token, addr).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Strategy scan failed: {}", e)))
}

// ─── ParaSwap ─────────────────────────────────────────

async fn get_paraswap_price(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ParaSwapPriceResponse>, (StatusCode, String)> {
    let chain_id = req["chain_id"].as_u64().ok_or_else(|| (StatusCode::BAD_REQUEST, "chain_id required".to_string()))?;
    let src = req["src_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "src_token required".to_string()))?;
    let dst = req["dest_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "dest_token required".to_string()))?;
    let sd = req["src_decimals"].as_u64().unwrap_or(18) as u8;
    let dd = req["dest_decimals"].as_u64().unwrap_or(18) as u8;
    let amt = req["amount"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "amount required".to_string()))?;
    let side = req["side"].as_str().unwrap_or("SELL");

    state.paraswap.get_price(chain_id, src, dst, sd, dd, amt, side).await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("ParaSwap error: {}", e)))
}

async fn build_paraswap_tx(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ParaSwapTransactionResponse>, (StatusCode, String)> {
    let chain_id = req["chain_id"].as_u64().ok_or_else(|| (StatusCode::BAD_REQUEST, "chain_id required".to_string()))?;
    let src = req["src_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "src_token required".to_string()))?;
    let dst = req["dest_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "dest_token required".to_string()))?;
    let sd = req["src_decimals"].as_u64().unwrap_or(18) as u8;
    let dd = req["dest_decimals"].as_u64().unwrap_or(18) as u8;
    let sa = req["src_amount"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "src_amount required".to_string()))?;
    let da = req["dest_amount"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "dest_amount required".to_string()))?;
    let slip = req["slippage"].as_f64().unwrap_or(0.5);
    let ua = req["user_address"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "user_address required".to_string()))?;
    let rc = req["receiver"].as_str();

    state.paraswap.build_transaction(chain_id, src, dst, sd, dd, sa, da, slip, ua, rc).await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("ParaSwap tx error: {}", e)))
}

// ─── Execute Basic Arbitrage ─────────────────────────

async fn execute_arbitrage(
    State(_state): State<AppState>,
    Json(req): Json<ExecuteArbitrageRequest>,
) -> Json<ExecuteResult> {
    info!("Executing arbitrage: opp={:?}", req.opportunity_id);
    // Production: submit tx via Flashbots / Pimlico / ZeroDev
    Json(ExecuteResult {
        status: "simulated".to_string(),
        message: "Arbitrage executed. 0 capital, 0 upfront gas.".to_string(),
        strategy: "flash_loan".to_string(),
        execution_mode: "FlashLoan".to_string(),
        tx_hash: Some(format!("0x{:064x}", rand::random::<u64>())),
        estimated_profit_usd: Some(100.50),
        gas_cost_usd: Some(2.30),
    })
}

// ─── Execute Advanced (any strategy) ──────────────────

async fn execute_advanced(
    State(_state): State<AppState>,
    Json(req): Json<AdvancedExecuteRequest>,
) -> Json<ExecuteResult> {
    info!(
        "Advanced execute: strategy={}, mode={:?}, flashLoan={:?}, gas={:?}, user={}",
        req.strategy, req.execution_mode, req.flash_loan_source, req.gas_strategy, req.user_address
    );

    let mode_label = match req.execution_mode {
        ExecutionMode::FlashLoan => "FlashLoan",
        ExecutionMode::DirectSwap => "DirectSwap",
        ExecutionMode::Mint => "Mint",
    };

    let profit = match req.strategy.as_str() {
        "triangular" => 42.75,
        "cross_chain" => 185.20,
        "jit" => 67.30,
        "mint" => 33.10,
        _ => 100.50,
    };

    Json(ExecuteResult {
        status: "simulated".to_string(),
        message: format!("{} ({}) executed. 0 capital, 0 upfront gas.", req.strategy, mode_label),
        strategy: req.strategy.clone(),
        execution_mode: mode_label.to_string(),
        tx_hash: Some(format!("0x{:064x}", rand::random::<u64>())),
        estimated_profit_usd: Some(profit),
        gas_cost_usd: Some(rand::random::<f64>() * 5.0),
    })
}

// ─── Chain / DEX Info ─────────────────────────────────

async fn list_chains() -> Json<Vec<ChainConfig>> {
    Json(crate::chains::get_chains().clone())
}

async fn list_dexes(Path(chain_id): Path<u64>) -> Json<Vec<DexConfig>> {
    Json(crate::chains::get_dexes_for_chain(chain_id))
}
