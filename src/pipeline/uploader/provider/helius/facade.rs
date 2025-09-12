#![allow(non_upper_case_globals)]
use crate::pipeline::uploader::provider::helius::client::HeliusClient;
use crate::util::alias::AResult;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::error;

pub fn helius_client() -> Arc<HeliusClient> {
    HeliusHolder.read().clone()
}

pub async fn send_helius_swqos(tx: &VersionedTransaction) -> AResult<()> {
    helius_client().send_transaction(tx).await?;
    Ok(())
}

static HeliusHolder: Lazy<RwLock<Arc<HeliusClient>>> = Lazy::new(|| {
    let client = Arc::new(HeliusClient::new(false));
    let client_clone = client.clone();

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = client_clone.ping().await {
                error!("Helius ping error: {}", e);
            }
        }
    });

    RwLock::new(client)
});
