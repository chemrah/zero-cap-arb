use crate::types::*;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tracing::{info, warn};

/// ParaSwap V5 API integration for optimal split-routing
pub struct ParaSwapClient {
    client: Client,
    base_url: &'static str,
    api_key: Option<String>,
}

/// ParaSwap V5 API endpoints
const PARASWAP_API_V5: &str = "https://api.paraswap.io/v5";
const PARASWAP_PARTNER: &str = "zerocaparb";

impl ParaSwapClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .user_agent("ZeroCapArb/1.0")
                .build()
                .expect("Failed to create HTTP client"),
            base_url: PARASWAP_API_V5,
            api_key,
        }
    }

    /// Get the best price for a token pair with full split-route data
    pub async fn get_price(
        &self,
        chain_id: u64,
        src_token: &str,
        dest_token: &str,
        src_decimals: u8,
        dest_decimals: u8,
        amount: &str,
        side: &str,
    ) -> Result<ParaSwapPriceResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut url = format!(
            "{}/prices/?srcToken={}&destToken={}&srcDecimals={}&destDecimals={}&amount={}&side={}&partner={}&network={}",
            self.base_url,
            src_token,
            dest_token,
            src_decimals,
            dest_decimals,
            amount,
            side,
            PARASWAP_PARTNER,
            chain_id,
        );

        if let Some(ref key) = self.api_key {
            url.push_str(&format!("&apiKey={}", key));
        }

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("ParaSwap price error {}: {}", status, body);
            return Err(format!("ParaSwap API error: {} - {}", status, body).into());
        }

        let raw: Value = resp.json().await?;

        // Parse the ParaSwap V5 response format
        let price_route = &raw["priceRoute"];

        let route_count = price_route["bestRoute"]
            .as_array()
            .map(|r| r.len())
            .unwrap_or(0);

        let mut routes = Vec::new();

        if let Some(best_routes) = price_route["bestRoute"].as_array() {
            for route_data in best_routes {
                let percent = route_data["percent"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let mut swaps = Vec::new();
                if let Some(swaps_arr) = route_data["swaps"].as_array() {
                    for swap in swaps_arr {
                        swaps.push(ParaSwapSwap {
                            exchange: swap["exchange"]
                                .as_str()
                                .unwrap_or("Unknown")
                                .to_string(),
                            src_amount: swap["srcAmount"]
                                .as_str()
                                .unwrap_or("0")
                                .to_string(),
                            dest_amount: swap["destAmount"]
                                .as_str()
                                .unwrap_or("0")
                                .to_string(),
                            percent: swap["percent"]
                                .as_str()
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or(0.0),
                            data: swap["data"].as_str().map(|s| s.to_string()),
                        });
                    }
                }

                routes.push(ParaSwapRoute {
                    src_token: src_token.to_string(),
                    src_decimals,
                    dest_token: dest_token.to_string(),
                    dest_decimals,
                    src_amount: route_data["srcAmount"]
                        .as_str()
                        .unwrap_or("0")
                        .to_string(),
                    dest_amount: route_data["destAmount"]
                        .as_str()
                        .unwrap_or("0")
                        .to_string(),
                    percentage: percent,
                    swaps,
                    exchange_fees: Vec::new(),
                });
            }
        }

        let gas_cost_usd = price_route["gasCostUSD"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok(ParaSwapPriceResponse {
            src_token: src_token.to_string(),
            dest_token: dest_token.to_string(),
            src_amount: amount.to_string(),
            dest_amount: price_route["destAmount"]
                .as_str()
                .unwrap_or("0")
                .to_string(),
            price_impact: price_route["priceImpact"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0),
            routes,
            gas_cost_usd,
        })
    }

    /// Build the full transaction calldata for executing the swap
    pub async fn build_transaction(
        &self,
        chain_id: u64,
        src_token: &str,
        dest_token: &str,
        src_decimals: u8,
        dest_decimals: u8,
        src_amount: &str,
        dest_amount: &str,
        slippage: f64,
        user_address: &str,
        receiver: Option<&str>,
    ) -> Result<ParaSwapTransactionResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/transactions/{}/{}?network={}",
            self.base_url, chain_id, src_token, chain_id,
        );

        let payload = serde_json::json!({
            "srcToken": src_token,
            "destToken": dest_token,
            "srcDecimals": src_decimals,
            "destDecimals": dest_decimals,
            "srcAmount": src_amount,
            "destAmount": dest_amount,
            "slippage": slippage,
            "priceRoute": {},
            "userAddress": user_address,
            "receiver": receiver.unwrap_or(user_address),
            "partner": PARASWAP_PARTNER,
        });

        info!(
            "Building ParaSwap tx: {} {} -> {} on chain {}",
            src_amount, src_token, dest_token, chain_id
        );

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("ParaSwap tx build error {}: {}", status, body);
            return Err(format!("ParaSwap tx error: {} - {}", status, body).into());
        }

        let raw: Value = resp.json().await?;

        Ok(ParaSwapTransactionResponse {
            from: raw["from"].as_str().unwrap_or("").to_string(),
            to: raw["to"].as_str().unwrap_or("").to_string(),
            value: raw["value"].as_str().unwrap_or("0").to_string(),
            data: raw["data"].as_str().unwrap_or("0x").to_string(),
            gas_price: raw["gasPrice"].as_str().unwrap_or("0").to_string(),
            gas: raw["gas"].as_str().unwrap_or("0").to_string(),
            chain_id,
        })
    }

    /// Get detailed token info with decimals
    pub async fn get_token_info(
        &self,
        chain_id: u64,
        token_address: &str,
    ) -> Result<(u8, String), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/tokens/{}/{}",
            self.base_url, chain_id, token_address
        );

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        let raw: Value = resp.json().await?;
        let decimals = raw["decimals"].as_u64().unwrap_or(18) as u8;
        let symbol = raw["symbol"].as_str().unwrap_or("Unknown").to_string();

        Ok((decimals, symbol))
    }

    /// Get all supported tokens on a chain from ParaSwap
    pub async fn get_supported_tokens(
        &self,
        chain_id: u64,
    ) -> Result<Vec<(String, String, u8)>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/tokens/{}", self.base_url, chain_id);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        let raw: Value = resp.json().await?;
        let mut tokens = Vec::new();

        if let Some(arr) = raw.as_array() {
            for token in arr {
                let address = token["address"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let symbol = token["symbol"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let decimals = token["decimals"]
                    .as_u64()
                    .unwrap_or(18) as u8;
                tokens.push((address, symbol, decimals));
            }
        }

        Ok(tokens)
    }
}
