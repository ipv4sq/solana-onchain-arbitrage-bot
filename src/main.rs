pub mod arb;
#[cfg(test)]
pub mod test;
pub mod util;

use crate::arb::pipeline::chain_subscriber::registrar::bootstrap_subscriber;
use solana_onchain_arbitrage_bot::arb::global;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        eprintln!("PANIC: {}", panic_info);
        eprintln!("Stack backtrace:\n{}", backtrace);
        std::process::exit(1);
    }));

    arb::util::logging::init()?;

    // Initialize database connection pool
    info!("Initializing database connection pool...");
    global::client::db::init_db().await?;
    info!("Database connection pool initialized");

    // Initialize blockhash holder with fresh blockhash
    info!("Initializing blockhash holder...");
    global::daemon::blockhash::initialize().await?;
    info!("Blockhash holder initialized");

    // 2. Start the SolanaMevBotOnchainListener
    let listener_handle = spawn_with_error_handling!("Subscriber", bootstrap_subscriber());

    // 3. Block until Ctrl+C
    info!("Press Ctrl+C to shutdown");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    listener_handle.abort();
    Ok(())
}
