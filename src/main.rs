pub mod arb;
mod bot;
mod config;
mod dex;
mod pools;
mod refresh;
mod server;
mod service;
#[cfg(test)]
pub mod test;
mod transaction;
pub mod util;

use arb::{global, program};
use clap::{App, Arg};
use std::fs;
use std::path::Path;
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create logs directory if it doesn't exist
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        fs::create_dir(logs_dir)?;
    }

    // Create a file for logging with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let log_file_path = format!("logs/bot_{}.log", timestamp);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path)?;

    // Create file layer for logging to file
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::sync::Arc::new(file))
        .with_ansi(false) // No color codes in file
        .with_line_number(true)
        .with_file(true);

    // Create console layer for logging to stdout
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true); // Color codes for console

    // Combine both layers with filtering
    // Default to info level, but exclude sqlx debug/trace logs
    let filter = tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".to_string()),
    );

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(filter)
        .init();

    info!("Starting Solana MEV Bot Listener");
    info!("Logs are being written to: {}", log_file_path);

    // Initialize blockhash holder with fresh blockhash
    info!("Initializing blockhash holder...");
    global::state::blockhash::initialize().await?;
    info!("Blockhash holder initialized");

    // 1. Trigger lazy initialization of MEV_TX_CONSUMER (just access it)
    let _ = &program::mev_bot::onchain_monitor::consumer::MEV_TX_CONSUMER;
    info!("MEV transaction consumer initialized");

    // 2. Start the SolanaMevBotOnchainListener
    let listener_handle = tokio::spawn(async move {
        if let Err(e) =
            program::mev_bot::onchain_monitor::producer::start_mev_bot_subscriber().await
        {
            tracing::error!("MEV bot subscriber error: {}", e);
        }
    });

    // 3. Block until Ctrl+C
    info!("Press Ctrl+C to shutdown");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    listener_handle.abort();
    Ok(())
}

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let subscriber = FmtSubscriber::builder()
//         .with_max_level(Level::INFO)
//         .finish();
//     tracing::subscriber::set_global_default(subscriber)
//         .expect("Failed to set global default subscriber");
//
//     info!("Starting Solana Onchain Bot");
//
//     let matches = App::new("Solana Onchain Arbitrage Bot")
//         .version("0.1.0")
//         .author("Cetipo")
//         .about("A simplified Solana onchain arbitrage bot")
//         .arg(
//             Arg::with_name("config")
//                 .short('c')
//                 .long("config")
//                 .value_name("FILE")
//                 .help("Sets a custom config file")
//                 .takes_value(true)
//                 .default_value("config.toml"),
//         )
//         .arg(
//             Arg::with_name("bot-only")
//                 .long("bot-only")
//                 .help("Run the bot directly without starting HTTP server")
//                 .takes_value(false),
//         )
//         .get_matches();
//
//     let config_path = matches.value_of("config").unwrap();
//     info!("Using config file: {}", config_path);
//
//     if matches.is_present("bot-only") {
//         info!("Running in bot-only mode (no HTTP server)");
//         bot::run_bot(config_path).await?;
//     } else {
//         info!("Starting HTTP server mode");
//         let config = config::Config::load(config_path)?;
//         server::run_server(config).await?;
//     }
//
//     Ok(())
// }
