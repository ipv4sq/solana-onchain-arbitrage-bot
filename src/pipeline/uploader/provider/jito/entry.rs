use crate::pipeline::uploader::provider::jito::client::JitoClient;
use crate::pipeline::uploader::provider::jito::types::{JitoBundleResponse, TipFloorData};
use crate::util::alias::AResult;
use crate::util::traits::pubkey::ToPubkey;
use anyhow::anyhow;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rand::Rng;
use reqwest::Client;
use serde_json::json;
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::system_instruction::transfer;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

pub(crate) const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

pub fn build_jito_tip_ix(payer: &Pubkey) -> Vec<Instruction> {
    let jito_tip_account = get_random_tip_account();
    let p75_jito_tip = get_jito_tips()
        .map(|t| t.landed_tips_75th_percentile)
        .unwrap_or(0.00001);
    let jito_tip_ix = transfer(
        &payer,
        &jito_tip_account,
        (p75_jito_tip * LAMPORTS_PER_SOL as f64) as u64,
    );
    vec![jito_tip_ix]
}

#[allow(non_upper_case_globals)]
static JitoClientHolder: Lazy<RwLock<Arc<JitoClient>>> = Lazy::new(|| {
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
    JitoClientHolder.read().clone()
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
