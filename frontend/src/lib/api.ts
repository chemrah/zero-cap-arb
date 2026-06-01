const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3001';

export interface ChainConfig {
  id: number;
  name: string;
  rpc_url: string;
  native_currency: string;
  explorer_url: string;
}

export interface DexConfig {
  name: string;
  address: string;
  chain_id: number;
}

// ─── Token Price ──────────────────────────────────────

export interface TokenPrice {
  token: string;
  token_address: string;
  chain_id: number;
  chain_name: string;
  dex_name: string;
  price_usd: number;
  liquidity_usd: number;
  timestamp: number;
}

export interface ChainSummary {
  chain_id: number;
  chain_name: string;
  dex_count: number;
  min_price: number;
  max_price: number;
  avg_price: number;
  spread_pct: number;
}

export interface AllPricesResponse {
  token: string;
  token_address: string;
  prices: TokenPrice[];
  chain_summary: ChainSummary[];
  scan_time_ms: number;
}

// ─── Simple Arbitrage ─────────────────────────────────

export interface ArbitrageOpportunity {
  id: string;
  token_symbol: string;
  token_address: string;
  buy_chain_id: number;
  buy_chain_name: string;
  buy_dex: string;
  buy_price_usd: number;
  sell_chain_id: number;
  sell_chain_name: string;
  sell_dex: string;
  sell_price_usd: number;
  spread_pct: number;
  estimated_profit_usd: number;
  liquidity_usd: number;
  timestamp: number;
}

export interface RadarScanResponse {
  token: string;
  opportunities: ArbitrageOpportunity[];
  scan_time_ms: number;
}

// ─── Advanced Arbitrage Types ─────────────────────────

export interface TriangularLeg {
  from_token: string;
  to_token: string;
  dex: string;
  rate: number;
  expected_output: string;
}

export interface TriangularOpportunity {
  id: string;
  chain_id: number;
  chain_name: string;
  legs: TriangularLeg[];
  start_token: string;
  end_token: string;
  net_profit_pct: number;
  estimated_profit_usd: number;
}

export interface CrossChainOpportunity {
  id: string;
  token: string;
  buy_chain_id: number;
  buy_chain_name: string;
  buy_price: number;
  sell_chain_id: number;
  sell_chain_name: string;
  sell_price: number;
  bridge_fee_usd: number;
  net_profit_pct: number;
  estimated_profit_usd: number;
}

export interface MintOpportunity {
  id: string;
  token: string;
  mint_platform: string;
  mint_cost_usd: number;
  market_price_usd: number;
  spread_pct: number;
  estimated_profit_usd: number;
}

export interface JitLiquidityOpportunity {
  id: string;
  token: string;
  pool: string;
  dex: string;
  chain_id: number;
  chain_name: string;
  expected_fee_usd: number;
  capital_required_usd: number;
}

export interface AllOpportunitiesResponse {
  token: string;
  simple_arbitrages: ArbitrageOpportunity[];
  triangular_arbitrages: TriangularOpportunity[];
  cross_chain_arbitrages: CrossChainOpportunity[];
  mint_opportunities: MintOpportunity[];
  jit_opportunities: JitLiquidityOpportunity[];
  scan_time_ms: number;
}

// ─── Velora Market API (ex-ParaSwap) ──────────────────

export interface VeloraRoute {
  src_token: string;
  dest_token: string;
  src_amount: string;
  dest_amount: string;
  percentage: number;
  exchange: string;
}

export interface VeloraPriceResponse {
  src_token: string;
  dest_token: string;
  src_amount: string;
  dest_amount: string;
  price_impact: number;
  routes: VeloraRoute[];
  gas_cost_usd: number;
  contract_address: string;
  token_transfer_proxy: string;
  version: string;
}

export interface VeloraTxResponse {
  from: string;
  to: string;
  value: string;
  data: string;
  gas_price: string;
  gas: string;
  chain_id: number;
}

// ─── Execution ────────────────────────────────────────

export interface ExecuteResult {
  status: string;
  message: string;
  strategy: string;
  execution_mode: string;
  tx_hash?: string;
  estimated_profit_usd?: number;
  gas_cost_usd?: number;
}

export interface HealthResponse {
  status: string;
  chains_connected: string[];
  uptime_secs: number;
}

// ─── API Client ───────────────────────────────────────

async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${endpoint}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!res.ok) {
    const err = await res.text();
    throw new Error(`API error ${res.status}: ${err}`);
  }
  return res.json();
}

export const api = {
  health: () => fetchApi<HealthResponse>('/api/health'),

  scanToken: (token: string, tokenAddress?: string) =>
    fetchApi<RadarScanResponse>('/api/scan', {
      method: 'POST',
      body: JSON.stringify({ token, token_address: tokenAddress }),
    }),

  getAllPrices: (token: string, tokenAddress?: string) =>
    fetchApi<AllPricesResponse>('/api/all-prices', {
      method: 'POST',
      body: JSON.stringify({ token, token_address: tokenAddress }),
    }),

  getAllOpportunities: (token: string, tokenAddress?: string) =>
    fetchApi<AllOpportunitiesResponse>('/api/all-opportunities', {
      method: 'POST',
      body: JSON.stringify({ token, token_address: tokenAddress }),
    }),

  getVeloraPrice: (params: {
    chain_id: number; src_token: string; dest_token: string;
    src_decimals: number; dest_decimals: number;
    amount: string; side: string;
  }) =>
    fetchApi<VeloraPriceResponse>('/api/velora/price', {
      method: 'POST', body: JSON.stringify(params),
    }),

  getVeloraSwap: (params: {
    chain_id: number; src_token: string; dest_token: string;
    src_decimals: number; dest_decimals: number;
    amount: string; side: string;
    user_address?: string; slippage?: number;
  }) =>
    fetchApi<Record<string, unknown>>('/api/velora/swap', {
      method: 'POST', body: JSON.stringify(params),
    }),

  buildVeloraTx: (params: {
    chain_id: number; src_token: string; dest_token: string;
    src_decimals: number; dest_decimals: number;
    src_amount: string; dest_amount: string;
    slippage: number; user_address: string;
    price_route?: Record<string, unknown>;
  }) =>
    fetchApi<VeloraTxResponse>('/api/velora/build-tx', {
      method: 'POST', body: JSON.stringify(params),
    }),

  submitDeltaOrder: (params: {
    chain_id: number; src_token: string; dest_token: string;
    amount: string; user_address: string;
  }) =>
    fetchApi<Record<string, unknown>>('/api/velora/delta', {
      method: 'POST', body: JSON.stringify(params),
    }),

  execute: (params: {
    strategy: string; execution_mode: string;
    flash_loan_source?: string; gas_strategy: string;
    user_address: string; token?: string; token_address?: string;
    amount?: string; chain_id?: number; opportunity_id?: string;
  }) =>
    fetchApi<ExecuteResult>('/api/execute/advanced', {
      method: 'POST', body: JSON.stringify(params),
    }),

  getChains: () => fetchApi<ChainConfig[]>('/api/chains'),
  getDexes: (chainId: number) => fetchApi<DexConfig[]>(`/api/dexes/${chainId}`),
};
