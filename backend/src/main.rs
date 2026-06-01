mod api;
mod chains;
mod paraswap_client;
mod radar_scanner;
mod types;
mod websocket;

use api::{build_router, AppState};
use paraswap_client::ParaSwapClient;
use radar_scanner::RadarScanner;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,zero_cap_arb_backend=debug")),
        )
        .init();

    // Load .env file if present
    dotenvy::dotenv().ok();

    info!("Starting Zero-Capital Arbitrage Backend...");

    // Verify chain configs load
    let chains = chains::get_chains();
    info!("Loaded {} chains", chains.len());

    for chain in chains {
        info!(
            "  Chain {}: {} (id: {}, RPC: {})",
            chain.id, chain.name, chain.id, chain.rpc_url
        );
        let dex_count = chains::get_dexes_for_chain(chain.id).len();
        info!("    {} DEXes configured", dex_count);
    }

    // Build shared state
    let state = AppState {
        scanner: Arc::new(RadarScanner::new()),
        paraswap: Arc::new(ParaSwapClient::new(
            std::env::var("PARASWAP_API_KEY").ok(),
        )),
        start_time: Instant::now(),
    };

    // Build router with CORS
    let app = build_router(state.clone())
        .layer(
            CorsLayer::permissive()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Zero-Cap Arbitrage API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Server shut down gracefully");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C signal"),
        _ = terminate => info!("Received SIGTERM signal"),
    }
}
