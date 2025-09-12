use crate::pipeline::uploader::provider::jito::client::{JitoClient, JITO_TIP_ACCOUNTS};
use crate::util::alias::AResult;
use crate::util::random::random_choose;
use crate::util::traits::pubkey::ToPubkey;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::system_instruction::transfer;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::error;

pub fn build_jito_tip_ix(payer: &Pubkey) -> (Vec<Instruction>, f64) {
    let tip_account = random_choose(&JITO_TIP_ACCOUNTS).to_pubkey();

    let p75_tip = jito_client()
        .get_latest_tip_amounts()
        .map(|t| t.landed_tips_75th_percentile)
        .unwrap_or(0.00001);

    let ix = transfer(
        &payer,
        &tip_account,
        (p75_tip * LAMPORTS_PER_SOL as f64) as u64,
    );
    (vec![ix], p75_tip)
}

pub fn jito_client() -> Arc<JitoClient> {
    JitoClientHolder.read().clone()
}

pub async fn send_bundle(tx: &VersionedTransaction) -> AResult<String> {
    jito_client().send_bundle(tx).await
}

pub async fn send_bundle_multi(txs: &[VersionedTransaction]) -> AResult<String> {
    jito_client().send_bundle_multi(txs).await
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
