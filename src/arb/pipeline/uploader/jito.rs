use crate::arb::util::alias::AResult;
use crate::arb::util::traits::pubkey::ToPubkey;
use anyhow::anyhow;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct JitoBundle {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct JitoBundleResponse {
    result: Option<String>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TipFloorData {
    #[allow(dead_code)]
    pub time: String,
    pub landed_tips_25th_percentile: f64,
    pub landed_tips_50th_percentile: f64,
    pub landed_tips_75th_percentile: f64,
    pub landed_tips_95th_percentile: f64,
    pub landed_tips_99th_percentile: f64,
    pub ema_landed_tips_50th_percentile: f64,
}

pub struct JitoClient {
    client: Client,
    base_url: String,
    latest_tip_amounts: RwLock<Option<TipFloorData>>,
}

impl JitoClient {
    pub fn new() -> Self {
        let base_url = "https://frankfurt.mainnet.block-engine.jito.wtf".to_string();

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            base_url,
            latest_tip_amounts: RwLock::new(None),
        }
    }

    pub async fn send_bundle(&self, tx: &VersionedTransaction) -> AResult<String> {
        self.send_bundle_multi(&[tx.clone()]).await
    }

    pub async fn send_bundle_multi(&self, txs: &[VersionedTransaction]) -> AResult<String> {
        if txs.is_empty() {
            return Err(anyhow!("No transactions provided for bundle"));
        }

        let mut signed_txs = Vec::new();
        for tx in txs {
            let serialized = bincode::serialize(tx)?;
            let encoded = BASE64_STANDARD.encode(&serialized);
            signed_txs.push(encoded);
        }

        let bundle_payload = json!({
            "jsonrpc": "2.0",
            "method": "sendBundle",
            "params": [
                signed_txs,
                {
                    "encoding": "base64"
                }
            ]
        });

        let bundle_url = format!("{}/api/v1/bundles", self.base_url);
        let response = self
            .client
            .post(&bundle_url)
            .header("Content-Type", "application/json")
            .json(&bundle_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Jito bundle submission failed with status {}: {}",
                status,
                error_text
            ));
        }

        let response_json: JitoBundleResponse = response.json().await?;

        if let Some(error) = response_json.error {
            return Err(anyhow!("Jito bundle error: {:?}", error));
        }

        let bundle_id = response_json
            .result
            .ok_or_else(|| anyhow!("No bundle ID in response"))?;

        info!("Bundle sent via Jito Frankfurt: {}", bundle_id);
        Ok(bundle_id)
    }

    pub async fn fetch_tip_amounts(&self) -> AResult<TipFloorData> {
        let tip_floor_url = "https://bundles.jito.wtf/api/v1/bundles/tip_floor";
        let response = self.client.get(tip_floor_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch tip amounts: {}",
                response.status()
            ));
        }

        let tip_data_array: Vec<TipFloorData> = response.json().await?;

        let tip_data = tip_data_array
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Empty tip data array"))?;

        *self.latest_tip_amounts.write() = Some(tip_data.clone());

        info!(
            "Fetched tip amounts - 50th: {:.6} SOL ({:.0} lamports), 95th: {:.6} SOL ({:.0} lamports)",
            tip_data.landed_tips_50th_percentile,
            tip_data.landed_tips_50th_percentile * 1e9,
            tip_data.landed_tips_95th_percentile,
            tip_data.landed_tips_95th_percentile * 1e9
        );
        Ok(tip_data)
    }

    pub fn get_latest_tip_amounts(&self) -> Option<TipFloorData> {
        self.latest_tip_amounts.read().clone()
    }

    pub fn get_latest_tip_amounts_lamports(&self) -> Option<(u64, u64, u64, u64, u64)> {
        self.latest_tip_amounts.read().as_ref().map(|data| {
            (
                (data.landed_tips_25th_percentile * 1e9) as u64,
                (data.landed_tips_50th_percentile * 1e9) as u64,
                (data.landed_tips_75th_percentile * 1e9) as u64,
                (data.landed_tips_95th_percentile * 1e9) as u64,
                (data.landed_tips_99th_percentile * 1e9) as u64,
            )
        })
    }

    pub fn get_random_tip_account() -> &'static str {
        let mut rng = rand::rng();
        let index = rng.random_range(0..JITO_TIP_ACCOUNTS.len());
        JITO_TIP_ACCOUNTS[index]
    }

    async fn periodic_tip_fetch(&self) -> AResult<()> {
        match self.fetch_tip_amounts().await {
            Ok(data) => {
                info!(
                    "Jito tip percentiles - 25th: {:.0}, 50th: {:.0}, 75th: {:.0}, 95th: {:.0}, 99th: {:.0} lamports",
                    data.landed_tips_25th_percentile * 1e9,
                    data.landed_tips_50th_percentile * 1e9,
                    data.landed_tips_75th_percentile * 1e9,
                    data.landed_tips_95th_percentile * 1e9,
                    data.landed_tips_99th_percentile * 1e9
                );
            }
            Err(e) => {
                warn!("Failed to fetch Jito tip amounts: {}", e);
            }
        }
        Ok(())
    }
}

static JITO_CLIENT_HOLDER: Lazy<RwLock<Arc<JitoClient>>> = Lazy::new(|| {
    let client = Arc::new(JitoClient::new());

    let client_clone = client.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = client_clone.periodic_tip_fetch().await {
                error!("Jito periodic tip fetch error: {}", e);
            }
        }
    });

    RwLock::new(client)
});

pub fn jito_client() -> Arc<JitoClient> {
    JITO_CLIENT_HOLDER.read().clone()
}

pub async fn send_bundle(tx: &VersionedTransaction) -> AResult<String> {
    jito_client().send_bundle(tx).await
}

pub async fn send_bundle_multi(txs: &[VersionedTransaction]) -> AResult<String> {
    jito_client().send_bundle_multi(txs).await
}

pub fn get_jito_tips() -> Option<TipFloorData> {
    jito_client().get_latest_tip_amounts()
}

pub fn get_random_tip_account() -> Pubkey {
    JitoClient::get_random_tip_account().to_pubkey()
}
