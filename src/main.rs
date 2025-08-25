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

use crate::arb::pipeline::pool_indexer::registrar::bootstrap_indexer;
use crate::arb::pipeline::swap_changes::account_monitor::subscriber::start_vault_monitor;
use arb::pipeline::pool_indexer;
use arb::{global, pipeline, program};
use clap::{App, Arg};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    arb::util::logging::init()?;

    // Initialize database connection
    if let Err(e) = global::db::init_db().await {
        log_error_with_backtrace!(warn, "Failed to initialize database connection", e);
        tracing::warn!("Database features will be unavailable");
    } else {
        info!("Database connection initialized");
    }

    // Initialize blockhash holder with fresh blockhash
    info!("Initializing blockhash holder...");
    global::state::blockhash::initialize().await?;
    info!("Blockhash holder initialized");

    // 2. Start the SolanaMevBotOnchainListener
    let listener_handle = spawn_with_error_handling!("MEV bot subscriber", bootstrap_indexer());

    let handle = spawn_with_error_handling!("Account subscriber", start_vault_monitor());

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
