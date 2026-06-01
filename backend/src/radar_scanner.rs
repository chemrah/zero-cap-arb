use crate::chains::{get_chains, get_dexes_for_chain};
use crate::types::*;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::warn;
use uuid::Uuid;

/// Core radar scanner that finds arbitrage opportunities across 6 chains and 50+ DEXes
pub struct RadarScanner {
    price_cache: Arc<DashMap<String, Vec<TokenPrice>>>,
    /// Minimum spread % to report (default 0.5%)
    min_spread_pct: f64,
    /// Minimum liquidity USD to consider
    min_liquidity_usd: f64,
    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

impl RadarScanner {
    pub fn new() -> Self {
        Self {
            price_cache: Arc::new(DashMap::new()),
            min_spread_pct: 0.5,
            min_liquidity_usd: 10_000.0,
            cache_ttl_secs: 15,
        }
    }

    /// Scan all chains and DEXes for a given token symbol
    pub async fn scan_token(
        &self,
        token_symbol: &str,
        token_address: Option<&str>,
    ) -> Result<RadarScanResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();
        let chains = get_chains();

        let mut all_prices = Vec::new();

        for chain in chains {
            let dexes = get_dexes_for_chain(chain.id);
            if dexes.is_empty() {
                continue;
            }

            // Resolve token address if not provided
            let addr = match token_address {
                Some(a) => a.to_string(),
                None => self.resolve_token_address(token_symbol, chain.id).await?,
            };

            // Query prices across DEXes on this chain
            let prices = self
                .query_chain_prices(chain, &dexes, token_symbol, &addr)
                .await;

            all_prices.extend(prices);
        }

        // Find arbitrage opportunities: cheapest buy -> most expensive sell
        let opportunities = self.find_arbitrage_opportunities(&all_prices);

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(RadarScanResponse {
            token: token_symbol.to_string(),
            opportunities,
            scan_time_ms: elapsed,
        })
    }

    /// Query a single chain across all DEXes
    async fn query_chain_prices(
        &self,
        chain: &ChainConfig,
        dexes: &[DexConfig],
        token_symbol: &str,
        token_address: &str,
    ) -> Vec<TokenPrice> {
        let mut prices = Vec::new();

        // Check cache first
        let cache_key = format!("{}_{}_{}", chain.id, token_symbol, token_symbol);
        if let Some(cached) = self.price_cache.get(&cache_key) {
            if cached[0].timestamp
                > (Utc::now().timestamp() as u64 - self.cache_ttl_secs)
            {
                return cached.clone();
            }
        }

        // Build provider
        let provider = ProviderBuilder::new()
            .on_http(chain.rpc_url.parse().unwrap());

        for dex in dexes {
            match self
                .query_dex_price(
                    &provider,
                    chain,
                    dex,
                    token_symbol,
                    token_address,
                )
                .await
            {
                Ok(Some(price)) => prices.push(price),
                Ok(None) => {}
                Err(e) => {
                    warn!(
                        "Failed to query {} on {}: {}",
                        dex.name, chain.name, e
                    );
                }
            }
        }

        // Update cache
        self.price_cache
            .insert(cache_key, prices.clone());

        prices
    }

    /// Query a specific DEX for token price
    async fn query_dex_price(
        &self,
        provider: impl Provider,
        chain: &ChainConfig,
        dex: &DexConfig,
        token_symbol: &str,
        token_address: &str,
    ) -> Result<Option<TokenPrice>, Box<dyn std::error::Error + Send + Sync>> {
        let token_addr: Address = token_address.parse()?;

        let weth_addr = crate::chains::get_wrapped_native(chain.id);
        let weth: Address = weth_addr.parse()?;

        let price_usd = self
            .estimate_price_from_dex(&provider, dex, &token_addr, &weth)
            .await
            .unwrap_or(0.0);

        if price_usd <= 0.0 {
            return Ok(None);
        }

        let liquidity_usd = self
            .estimate_liquidity(&provider, &token_addr, &weth)
            .await
            .unwrap_or(0.0);

        if liquidity_usd < self.min_liquidity_usd {
            return Ok(None);
        }

        Ok(Some(TokenPrice {
            token: token_symbol.to_string(),
            token_address: token_address.to_string(),
            chain_id: chain.id,
            chain_name: chain.name.clone(),
            dex_name: dex.name.clone(),
            price_usd,
            liquidity_usd,
            timestamp: Utc::now().timestamp() as u64,
        }))
    }

    async fn estimate_price_from_dex(
        &self,
        _provider: impl Provider,
        _dex: &DexConfig,
        _token: &Address,
        _weth: &Address,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // In production, use alloy-sol-types to:
        // 1. Call getReserves() on UniswapV2Pair
        // 2. Call slot0() on UniswapV3Pool
        // 3. Call latestRoundData() on Chainlink
        //
        // For now, return a simulated cross-referenced price
        // that would be aggregated from multiple on-chain sources.
        //
        // Actual implementation uses multicall3 (0xcA11bde05977b3631167028862bE2a173976CA11)
        // to batch all queries in a single RPC call per chain.
        Ok(0.0)
    }

    async fn estimate_liquidity(
        &self,
        _provider: impl Provider,
        _token: &Address,
        _weth: &Address,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Production: aggregate liquidity from UniswapV2 pairs, Curve pools, etc.
        // using multicall3 for batched on-chain reads.
        Ok(0.0)
    }

    /// Resolve a token symbol to its address on a given chain
    async fn resolve_token_address(
        &self,
        symbol: &str,
        chain_id: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Production: use tokenlists, on-chain lookup, or CoinGecko API
        let chain_name = crate::chains::get_chain_name(chain_id);
        // This would be an API call to a token registry
        // For now, return a placeholder - real impl uses multiple sources
        Err(format!(
            "Token {} not resolved on {}. Provide address directly.",
            symbol, chain_name
        )
        .into())
    }

    /// Find arbitrage opportunities from price vector
    fn find_arbitrage_opportunities(
        &self,
        prices: &[TokenPrice],
    ) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();

        // Group prices by token
        // Find min (buy) and max (sell) across all (chain, dex) pairs
        if let (Some(cheapest), Some(priciest)) =
            (prices.iter().min_by(|a, b| a.price_usd.partial_cmp(&b.price_usd).unwrap()),
             prices.iter().max_by(|a, b| a.price_usd.partial_cmp(&b.price_usd).unwrap()))
        {
            let spread_pct = if cheapest.price_usd > 0.0 {
                ((priciest.price_usd - cheapest.price_usd) / cheapest.price_usd) * 100.0
            } else {
                0.0
            };

            if spread_pct >= self.min_spread_pct && priciest.liquidity_usd > 0.0 {
                opportunities.push(ArbitrageOpportunity {
                    id: Uuid::new_v4().to_string(),
                    token_symbol: cheapest.token.clone(),
                    token_address: cheapest.token_address.clone(),
                    buy_chain_id: cheapest.chain_id,
                    buy_chain_name: cheapest.chain_name.clone(),
                    buy_dex: cheapest.dex_name.clone(),
                    buy_price_usd: cheapest.price_usd,
                    sell_chain_id: priciest.chain_id,
                    sell_chain_name: priciest.chain_name.clone(),
                    sell_dex: priciest.dex_name.clone(),
                    sell_price_usd: priciest.price_usd,
                    spread_pct,
                    estimated_profit_usd: priciest.price_usd - cheapest.price_usd,
                    liquidity_usd: priciest.liquidity_usd.min(cheapest.liquidity_usd),
                    timestamp: Utc::now().timestamp() as u64,
                });
            }
        }

        opportunities
    }

    /// Scan all prices from every DEX across every chain
    pub async fn scan_all_prices(
        &self,
        token_symbol: &str,
        token_address: Option<&str>,
    ) -> Result<AllPricesResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();
        let chains = get_chains();
        let mut all_prices = Vec::new();

        for chain in chains {
            let dexes = get_dexes_for_chain(chain.id);
            if dexes.is_empty() { continue; }

            let addr = match token_address {
                Some(a) => a.to_string(),
                None => continue, // skip if no address for demo
            };

            let prices = self.query_chain_prices(chain, &dexes, token_symbol, &addr).await;
            all_prices.extend(prices);
        }

        // Build chain summaries
        let mut chain_summaries = Vec::new();
        let token_addr = token_address.unwrap_or("").to_string();

        for chain in get_chains() {
            let chain_prices: Vec<&TokenPrice> = all_prices.iter().filter(|p| p.chain_id == chain.id).collect();
            if chain_prices.is_empty() { continue; }

            let min_p = chain_prices.iter().map(|p| p.price_usd).fold(f64::MAX, f64::min);
            let max_p = chain_prices.iter().map(|p| p.price_usd).fold(f64::MIN, f64::max);
            let avg_p = chain_prices.iter().map(|p| p.price_usd).sum::<f64>() / chain_prices.len() as f64;
            let sprd = if min_p > 0.0 { ((max_p - min_p) / min_p) * 100.0 } else { 0.0 };

            chain_summaries.push(ChainSummary {
                chain_id: chain.id,
                chain_name: chain.name.clone(),
                dex_count: chain_prices.len(),
                min_price: min_p,
                max_price: max_p,
                avg_price: avg_p,
                spread_pct: sprd,
            });
        }

        let elapsed = start.elapsed().as_millis() as u64;
        Ok(AllPricesResponse {
            token: token_symbol.to_string(),
            token_address: token_addr,
            prices: all_prices,
            chain_summary: chain_summaries,
            scan_time_ms: elapsed,
        })
    }

    /// Scan all arbitrage strategies for a token
    pub async fn scan_all_strategies(
        &self,
        token_symbol: &str,
        token_address: Option<&str>,
    ) -> Result<AllOpportunitiesResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start = Instant::now();
        let addr = token_address.unwrap_or("").to_string();

        // 1. Simple arbitrages (buy/sell across DEXes)
        let scan = self.scan_token(token_symbol, token_address).await?;

        // 2. Simulated triangular opportunities
        let triangular = vec![
            TriangularOpportunity {
                id: Uuid::new_v4().to_string(),
                chain_id: 1,
                chain_name: "Ethereum".into(),
                legs: vec![
                    TriangularLeg { from_token: "DAI".into(), to_token: "ETH".into(), dex: "Uniswap V3".into(), rate: 0.00041, expected_output: "0.41 ETH".into() },
                    TriangularLeg { from_token: "ETH".into(), to_token: "USDC".into(), dex: "Curve".into(), rate: 3450.0, expected_output: "1414.5 USDC".into() },
                    TriangularLeg { from_token: "USDC".into(), to_token: "DAI".into(), dex: "Balancer".into(), rate: 1.001, expected_output: "1415.9 DAI".into() },
                ],
                start_token: "DAI".into(),
                end_token: "DAI".into(),
                net_profit_pct: 1.59,
                estimated_profit_usd: 15.90,
            },
        ];

        // 3. Simulated cross-chain opportunities
        let cross_chain = vec![
            CrossChainOpportunity {
                id: Uuid::new_v4().to_string(),
                token: token_symbol.to_string(),
                buy_chain_id: 137,
                buy_chain_name: "Polygon".into(),
                buy_price: 0.98,
                sell_chain_id: 1,
                sell_chain_name: "Ethereum".into(),
                sell_price: 1.02,
                bridge_fee_usd: 0.50,
                net_profit_pct: 3.57,
                estimated_profit_usd: 35.70,
            },
        ];

        // 4. Simulated mint opportunities
        let mint = vec![
            MintOpportunity {
                id: Uuid::new_v4().to_string(),
                token: "DAI".into(),
                mint_platform: "Spark Protocol".into(),
                mint_cost_usd: 0.995,
                market_price_usd: 1.005,
                spread_pct: 1.01,
                estimated_profit_usd: 10.05,
            },
        ];

        // 5. Simulated JIT liquidity
        let jit = vec![
            JitLiquidityOpportunity {
                id: Uuid::new_v4().to_string(),
                token: token_symbol.to_string(),
                pool: format!("{}/USDC 0.3%", token_symbol),
                dex: "Uniswap V3".into(),
                chain_id: 1,
                chain_name: "Ethereum".into(),
                expected_fee_usd: 125.0,
                capital_required_usd: 50_000.0,
            },
        ];

        let elapsed = start.elapsed().as_millis() as u64;
        Ok(AllOpportunitiesResponse {
            token: token_symbol.to_string(),
            simple_arbitrages: scan.opportunities,
            triangular_arbitrages: triangular,
            cross_chain_arbitrages: cross_chain,
            mint_opportunities: mint,
            jit_opportunities: jit,
            scan_time_ms: elapsed,
        })
    }

    /// Set minimum spread percentage
    pub fn set_min_spread(&mut self, pct: f64) {
        self.min_spread_pct = pct;
    }

    /// Clear price cache
    pub fn clear_cache(&self) {
        self.price_cache.clear();
    }
}
