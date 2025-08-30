use crate::arb::pipeline::chain_subscriber::mev_bot_tx_subscriber::start_mev_bot_subscriber;
use crate::unit_ok;
use anyhow::Result;

pub async fn bootstrap_subscriber() -> Result<()> {
    tokio::spawn(async move {
        let _ = start_mev_bot_subscriber().await;
    });
    unit_ok!()
}
