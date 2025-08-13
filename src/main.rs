mod bot;
mod config;
mod constants;
mod dex;
mod pools;
mod refresh;
mod server;
mod service;
mod transaction;
#[cfg(test)]
pub mod test;
pub mod util;

use clap::{App, Arg};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting Solana Onchain Bot");

    let matches = App::new("Solana Onchain Arbitrage Bot")
        .version("0.1.0")
        .author("Cetipo")
        .about("A simplified Solana onchain arbitrage bot")
        .arg(
            Arg::with_name("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .default_value("config.toml"),
        )
        .arg(
            Arg::with_name("bot-only")
                .long("bot-only")
                .help("Run the bot directly without starting HTTP server")
                .takes_value(false),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap();
    info!("Using config file: {}", config_path);

    if matches.is_present("bot-only") {
        info!("Running in bot-only mode (no HTTP server)");
        bot::run_bot(config_path).await?;
    } else {
        info!("Starting HTTP server mode");
        let config = config::Config::load(config_path)?;
        server::run_server(config).await?;
    }

    Ok(())
}
