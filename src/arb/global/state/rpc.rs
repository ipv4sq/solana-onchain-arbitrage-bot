use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::convention::chain::transaction::Transaction;
use crate::arb::convention::chain::util::simulation::SimulationResult;
use crate::arb::convention::pool::util::ata;
use crate::arb::global::constant::token_program::TokenProgram;
use crate::constants::helpers::{ToPubkey, ToSignature};
use anyhow::Result;
use instruction::create_associated_token_account_idempotent;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_client::nonblocking::rpc_client;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::VersionedTransaction;
use spl_associated_token_account::instruction;
use std::cmp::min;
use std::sync::Arc;
use tracing::info;

pub async fn send_tx_with_retry(tx: &VersionedTransaction, max_retries: u64) -> Result<Signature> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();
    info!("Transaction size after compilation: {} bytes", tx_size);

    rpc_client()
        .send_transaction_with_config(
            tx,
            solana_client::rpc_config::RpcSendTransactionConfig {
                skip_preflight: true,
                max_retries: Some(max_retries as usize),
                preflight_commitment: Some(CommitmentLevel::Confirmed),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))
}

pub async fn simulate_tx_with_retry(
    tx: &VersionedTransaction,
    _max_retries: u64,
) -> Result<SimulationResult> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();
    info!("Transaction size after compilation: {} bytes", tx_size);

    let response = rpc_client().simulate_transaction(tx).await?;
    Ok(SimulationResult::from(&response.value))
}

pub async fn fetch_tx(signature: &str) -> Result<Transaction> {
    rpc_client()
        .get_transaction_with_config(&signature.to_sig(), json_config())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))?
        .to_unified()
}

pub fn fetch_tx_sync(client: &RpcClient, signature: &str) -> Result<Transaction> {
    client
        .get_transaction_with_config(&signature.to_sig(), json_config())
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))?
        .to_unified()
}

static RPC_HOLDER: Lazy<RwLock<Arc<rpc_client::RpcClient>>> = Lazy::new(|| {
    let url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    RwLock::new(Arc::new(rpc_client::RpcClient::new(url)))
});

pub fn rpc_client() -> Arc<rpc_client::RpcClient> {
    RPC_HOLDER.read().clone()
}

fn json_config() -> RpcTransactionConfig {
    RpcTransactionConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
        commitment: None,
        max_supported_transaction_version: Some(0),
    }
}

pub async fn ensure_mint_account_exists(mint: &Pubkey, wallet: &Keypair) -> Result<bool> {
    let owner = &wallet.pubkey();
    let mint_owner = rpc_client().get_account(mint).await?.owner;
    if mint_owner != TokenProgram::TOKEN_2022 && mint_owner != TokenProgram::SPL_TOKEN {
        return Err(anyhow::anyhow!(
            "mint owner should be Token2022 or SPL Token program but instead it's: {}",
            mint_owner
        ));
    }

    let mint_account = ata(owner, mint, &mint_owner);
    let mint_account_exists = rpc_client().get_account(&mint_account).await.is_ok();
    if !mint_account_exists {
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[
                ComputeBudgetInstruction::set_compute_unit_price(1_000),
                ComputeBudgetInstruction::set_compute_unit_limit(30_000),
                create_associated_token_account_idempotent(owner, owner, mint, &mint_owner),
            ],
            Some(owner),
            &[wallet],
            rpc_client().get_latest_blockhash().await?,
        );
        let signature = rpc_client()
            .send_and_confirm_transaction_with_spinner(&tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;
        info!(
            "Created token account for mint: {} owner: {} tx: {}",
            mint, owner, signature
        );
    } else {
        info!("Mint account exists mint: {} owner: {}", mint, owner);
    }
    Ok(true)
}

// 仅测试使用：替换全局句柄
#[cfg(test)]
pub fn _set_test_client(client: rpc_client::RpcClient) {
    *RPC_HOLDER.write() = Arc::new(client);
}
