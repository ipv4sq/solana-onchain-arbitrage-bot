use crate::arb::util::alias::AResult;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use reqwest::Client;
use serde_json::json;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

pub struct HeliusClient {
    client: Client,
    url: String,
    ping_url: String,
}

impl HeliusClient {
    pub fn new() -> Self {
        let url = "http://fra-sender.helius-rpc.com/fast?swqos_only=true".to_string();
        let ping_url = "http://fra-sender.helius-rpc.com/ping".to_string();
        Self {
            client: Client::new(),
            url,
            ping_url,
        }
    }

    pub async fn send_transaction(&self, tx: &VersionedTransaction) -> AResult<Signature> {
        let serialized_tx = bincode::serialize(tx)?;
        let encoded_tx = BASE64_STANDARD.encode(&serialized_tx);

        let request_body = json!({
            "jsonrpc": "2.0",
            "id": chrono::Utc::now().timestamp_millis().to_string(),
            "method": "sendTransaction",
            "params": [
                encoded_tx,
                {
                    "encoding": "base64",
                    "skipPreflight": true,
                    "maxRetries": 0
                }
            ]
        });

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        if let Some(error) = response_json.get("error") {
            return Err(anyhow::anyhow!("Transaction error: {:?}", error));
        }

        let signature_str = response_json["result"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get signature from response"))?;

        let signature = Signature::from_str(signature_str)?;
        info!("Transaction sent via Helius: {}", signature);
        Ok(signature)
    }

    async fn ping(&self) -> AResult<()> {
        let response = self
            .client
            .get(&self.ping_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if response.status().is_success() {
            info!("Helius ping successful");
        } else {
            warn!("Helius ping failed with status: {}", response.status());
        }
        Ok(())
    }
}

static HELIUS_CLIENT_HOLDER: Lazy<RwLock<Arc<HeliusClient>>> = Lazy::new(|| {
    let client = Arc::new(HeliusClient::new());

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

pub fn helius_client() -> Arc<HeliusClient> {
    HELIUS_CLIENT_HOLDER.read().clone()
}

pub async fn sender(tx: &VersionedTransaction) -> AResult<()> {
    helius_client().send_transaction(tx).await?;
    Ok(())
}
