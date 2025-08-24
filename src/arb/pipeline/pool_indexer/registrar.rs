use crate::arb::pipeline::pool_indexer::mev_bot::producer::start_mev_bot_subscriber;
use crate::empty_ok;
use anyhow::Result;

pub async fn bootstrap_indexer() -> Result<()> {
    tokio::spawn(async move {
        let _ = start_mev_bot_subscriber().await;
    });
    empty_ok!()
}
