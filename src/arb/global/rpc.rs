use crate::arb::chain::mapper::traits::ToUnified;
use crate::arb::chain::transaction::Transaction;
use crate::arb::chain::util::simulation::SimulationResult;
use crate::constants::helpers::ToSignature;
use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_client::nonblocking::rpc_client;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::Arc;

pub async fn send_tx_with_retry(tx: &VersionedTransaction, max_retries: u64) -> Result<Signature> {
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
// 仅测试使用：替换全局句柄
#[cfg(test)]
pub fn _set_test_client(client: rpc_client::RpcClient) {
    *RPC_HOLDER.write() = Arc::new(client);
}
