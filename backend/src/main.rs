mod api;
mod chains;
mod radar_scanner;
mod types;
mod velora_client;
mod websocket;

use api::{build_router, AppState};
use radar_scanner::RadarScanner;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;
use velora_client::VeloraClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    dotenvy::dotenv().ok();

    info!("Starting Zero-Capital Arbitrage Backend (Velora engine)...");

    let chains = chains::get_chains();
    info!("Loaded {} chains", chains.len());
    for chain in chains {
        let dex_count = chains::get_dexes_for_chain(chain.id).len();
        info!("  Chain {}: {} (id: {}, {} DEXes)", chain.id, chain.name, chain.id, dex_count);
    }

    let state = AppState {
        scanner: Arc::new(RadarScanner::new()),
        velora: Arc::new(VeloraClient::new(std::env::var("VELORA_API_KEY").ok())),
        start_time: Instant::now(),
    };

    let app = build_router(state.clone()).layer(CorsLayer::permissive());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string()).parse::<u16>().unwrap_or(3001);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Zero-Cap Arbitrage API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Server shut down");
}

async fn shutdown_signal() {
    let ctrl_c = async { signal::ctrl_c().await.expect("Failed to install Ctrl+C handler"); };
    #[cfg(unix)]
    let terminate = async { signal::unix::signal(signal::unix::SignalKind::terminate()).expect("...").recv().await; };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C"),
        _ = terminate => info!("Received SIGTERM"),
    }
}
