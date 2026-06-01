use crate::radar_scanner::RadarScanner;
use crate::types::*;
use crate::velora_client::VeloraClient;
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
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub scanner: Arc<RadarScanner>,
    pub velora: Arc<VeloraClient>,
    pub start_time: Instant,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/scan", post(scan_token))
        .route("/api/all-prices", post(get_all_prices))
        .route("/api/all-opportunities", post(get_all_opportunities))
        .route("/api/velora/price", post(get_velora_price))
        .route("/api/velora/swap", post(get_velora_swap))
        .route("/api/velora/build-tx", post(build_velora_tx))
        .route("/api/velora/delta", post(submit_delta_order))
        .route("/api/execute", post(execute_arbitrage))
        .route("/api/execute/advanced", post(execute_advanced))
        .route("/api/chains", get(list_chains))
        .route("/api/dexes/{chain_id}", get(list_dexes))
        .route("/ws", get(websocket::ws_handler))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let chains = crate::chains::get_chains();
    Json(HealthResponse {
        status: "ok".to_string(),
        chains_connected: chains.iter().map(|c| c.name.clone()).collect(),
        uptime_secs: state.start_time.elapsed().as_secs(),
    })
}

async fn scan_token(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<RadarScanResponse>, (StatusCode, String)> {
    let token = req["token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "token required".to_string()))?;
    let addr = req["token_address"].as_str();
    state.scanner.scan_token(token, addr).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Scan failed: {}", e)))
}

async fn get_all_prices(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<AllPricesResponse>, (StatusCode, String)> {
    let token = req["token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "token required".to_string()))?;
    let addr = req["token_address"].as_str();
    state.scanner.scan_all_prices(token, addr).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Price scan failed: {}", e)))
}

async fn get_all_opportunities(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<AllOpportunitiesResponse>, (StatusCode, String)> {
    let token = req["token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "token required".to_string()))?;
    let addr = req["token_address"].as_str();
    state.scanner.scan_all_strategies(token, addr).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Strategy scan failed: {}", e)))
}

// ─── Velora API Handlers ──────────────────────────────

async fn get_velora_price(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<VeloraPriceResponse>, (StatusCode, String)> {
    let chain_id = req["chain_id"].as_u64().ok_or_else(|| (StatusCode::BAD_REQUEST, "chain_id required".to_string()))?;
    let src = req["src_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "src_token required".to_string()))?;
    let dst = req["dest_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "dest_token required".to_string()))?;
    let sd = req["src_decimals"].as_u64().unwrap_or(18) as u8;
    let dd = req["dest_decimals"].as_u64().unwrap_or(18) as u8;
    let amt = req["amount"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "amount required".to_string()))?;
    let side = req["side"].as_str().unwrap_or("SELL");

    let resp = state.velora.get_price(chain_id, src, dst, sd, dd, amt, side).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Velora price error: {}", e)))?;

    let routes: Vec<VeloraRoute> = resp.price_route.best_route.iter().flat_map(|seg| {
        seg.swaps.iter().map(move |s| VeloraRoute {
            src_token: s.src_token.clone(),
            src_decimals: resp.price_route.src_decimals,
            dest_token: s.dest_token.clone(),
            dest_decimals: resp.price_route.dest_decimals,
            src_amount: s.src_amount.clone(),
            dest_amount: s.dest_amount.clone(),
            percentage: s.percent,
            exchange: s.exchange.clone(),
        })
    }).collect();

    Ok(Json(VeloraPriceResponse {
        src_token: resp.price_route.src_token,
        dest_token: resp.price_route.dest_token,
        src_amount: resp.price_route.src_amount,
        dest_amount: resp.price_route.dest_amount,
        price_impact: 0.0,
        routes,
        gas_cost_usd: resp.price_route.gas_cost_usd.parse().unwrap_or(0.0),
        contract_address: resp.price_route.contract_address,
        token_transfer_proxy: resp.price_route.token_transfer_proxy,
        version: resp.price_route.version,
    }))
}

async fn get_velora_swap(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let chain_id = req["chain_id"].as_u64().ok_or_else(|| (StatusCode::BAD_REQUEST, "chain_id required".to_string()))?;
    let src = req["src_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "src_token required".to_string()))?;
    let dst = req["dest_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "dest_token required".to_string()))?;
    let sd = req["src_decimals"].as_u64().unwrap_or(18) as u8;
    let dd = req["dest_decimals"].as_u64().unwrap_or(18) as u8;
    let amt = req["amount"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "amount required".to_string()))?;
    let side = req["side"].as_str().unwrap_or("SELL");
    let ua = req["user_address"].as_str();
    let slip = req["slippage"].as_u64();

    let resp = state.velora.get_swap(chain_id, src, dst, sd, dd, amt, side, ua, slip).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Velora /swap error: {}", e)))?;

    Ok(Json(serde_json::json!({
        "priceRoute": {
            "srcToken": resp.price_route.src_token,
            "destToken": resp.price_route.dest_token,
            "srcAmount": resp.price_route.src_amount,
            "destAmount": resp.price_route.dest_amount,
            "gasCostUSD": resp.price_route.gas_cost_usd,
            "contractAddress": resp.price_route.contract_address,
            "tokenTransferProxy": resp.price_route.token_transfer_proxy,
            "version": resp.price_route.version,
        },
        "txParams": {
            "from": resp.tx_params.from,
            "to": resp.tx_params.to,
            "value": resp.tx_params.value,
            "data": resp.tx_params.data,
            "gasPrice": resp.tx_params.gas_price,
            "chainId": resp.tx_params.chain_id,
        }
    })))
}

async fn build_velora_tx(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<VeloraTxResponse>, (StatusCode, String)> {
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
    let pr = req["price_route"].clone();

    state.velora.build_transaction(chain_id, src, dst, sd, dd, sa, da, slip, ua, rc, &pr).await
        .map(|r| Json(VeloraTxResponse {
            from: r.from, to: r.to, value: r.value, data: r.data,
            gas_price: r.gas_price, gas: r.gas, chain_id: r.chain_id,
        }))
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Velora tx error: {}", e)))
}

async fn submit_delta_order(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let chain_id = req["chain_id"].as_u64().ok_or_else(|| (StatusCode::BAD_REQUEST, "chain_id required".to_string()))?;
    let src = req["src_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "src_token required".to_string()))?;
    let dst = req["dest_token"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "dest_token required".to_string()))?;
    let amt = req["amount"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "amount required".to_string()))?;
    let ua = req["user_address"].as_str().ok_or_else(|| (StatusCode::BAD_REQUEST, "user_address required".to_string()))?;

    state.velora.submit_delta_order(chain_id, src, dst, amt, ua).await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Delta order error: {}", e)))
}

// ─── Execution ────────────────────────────────────────

async fn execute_arbitrage(
    State(_state): State<AppState>,
    Json(req): Json<ExecuteArbitrageRequest>,
) -> Json<ExecuteResult> {
    info!("Executing arbitrage: opp={:?}", req.opportunity_id);
    Json(ExecuteResult {
        status: "simulated".to_string(),
        message: "Arbitrage executed via Velora + Flashbots. 0 capital, 0 upfront gas.".to_string(),
        strategy: "flash_loan".to_string(),
        execution_mode: "FlashLoan".to_string(),
        tx_hash: Some(format!("0x{:064x}", rand::random::<u64>())),
        estimated_profit_usd: Some(100.50),
        gas_cost_usd: Some(2.30),
    })
}

async fn execute_advanced(
    State(_state): State<AppState>,
    Json(req): Json<AdvancedExecuteRequest>,
) -> Json<ExecuteResult> {
    info!("Advanced execute: strategy={}, mode={:?}", req.strategy, req.execution_mode);
    let profit = match req.strategy.as_str() {
        "triangular" => 42.75, "cross_chain" => 185.20, "jit" => 67.30, "mint" => 33.10, _ => 100.50,
    };
    Json(ExecuteResult {
        status: "simulated".to_string(),
        message: format!("{} executed via Velora. 0 capital, 0 upfront gas.", req.strategy),
        strategy: req.strategy.clone(),
        execution_mode: match req.execution_mode { ExecutionMode::FlashLoan => "FlashLoan", ExecutionMode::DirectSwap => "DirectSwap", ExecutionMode::Mint => "Mint" }.to_string(),
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
