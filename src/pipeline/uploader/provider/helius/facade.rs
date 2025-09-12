#![allow(non_upper_case_globals)]
use crate::pipeline::uploader::provider::helius::client::{HeliusClient, HELIUS_TIP_ACCOUNTS};
use crate::unit_ok;
use crate::util::alias::AResult;
use crate::util::random::random_choose;
use crate::util::traits::pubkey::ToPubkey;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::transfer;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::error;

pub fn build_helius_jito_tip_ix(payer: &Pubkey) -> (Vec<Instruction>, f64) {
    let tip_account = random_choose(&HELIUS_TIP_ACCOUNTS).to_pubkey();
    let minimum_tip = 0.001;
    let ix = transfer(
        &payer,
        &tip_account,
        (minimum_tip * LAMPORTS_PER_SOL as f64) as u64,
    );
    (vec![ix], minimum_tip)
}

pub fn build_helius_swqos_tip_ix(payer: &Pubkey) -> (Vec<Instruction>, f64) {
    let tip_account = random_choose(&HELIUS_TIP_ACCOUNTS).to_pubkey();
    let minimum_tip = 0.0005;
    let ix = transfer(
        &payer,
        &tip_account,
        (minimum_tip * LAMPORTS_PER_SOL as f64) as u64,
    );
    (vec![ix], minimum_tip)
}

pub async fn send_helius_swqos(tx: &VersionedTransaction) -> AResult<()> {
    HeliusSwqosHolder
        .read()
        .clone()
        .send_transaction(tx)
        .await?;
    unit_ok!()
}

pub async fn send_helius_jito(tx: &VersionedTransaction) -> AResult<()> {
    HeliusJitoHolder.read().clone().send_transaction(tx).await?;
    unit_ok!()
}

static HeliusJitoHolder: Lazy<RwLock<Arc<HeliusClient>>> = Lazy::new(|| {
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

static HeliusSwqosHolder: Lazy<RwLock<Arc<HeliusClient>>> = Lazy::new(|| {
    let client = Arc::new(HeliusClient::new(true));
    let client_clone = client.clone();

    //noinspection DuplicatedCode
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
