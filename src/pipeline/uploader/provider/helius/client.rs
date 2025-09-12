use crate::util::alias::AResult;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use reqwest::Client;
use serde_json::json;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use std::str::FromStr;
use std::time::Duration;
use tracing::{info, warn};

pub(crate) const HELIUS_TIP_ACCOUNTS: &[&str] = &[
    "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE",
    "D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ",
    "9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta",
    "5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn",
    "2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD",
    "2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ",
    "wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF",
    "3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT",
    "4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey",
    "4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or",
];

pub struct HeliusClient {
    client: Client,
    url: String,
    ping_url: String,
}

impl HeliusClient {
    pub fn new(swqos_only: bool) -> Self {
        Self {
            client: Client::new(),
            url: if swqos_only {
                "http://fra-sender.helius-rpc.com/fast?swqos_only=true"
            } else {
                "http://fra-sender.helius-rpc.com/fast"
            }
            .to_string(),
            ping_url: "http://fra-sender.helius-rpc.com/ping".to_string(),
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

    pub async fn ping(&self) -> AResult<()> {
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
