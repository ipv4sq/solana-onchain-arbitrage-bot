use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::convention::chain::util::simulation::SimulationResult;
use crate::arb::convention::chain::Transaction;
use crate::arb::sdk::solana_rpc::rpc;
use crate::arb::util::traits::signature::ToSignature;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use tracing::info;

pub async fn send_tx_with_retry(
    tx: &VersionedTransaction,
    max_retries: u64,
) -> anyhow::Result<Signature> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();
    info!("Transaction size after compilation: {} bytes", tx_size);

    rpc::rpc_client()
        .send_transaction_with_config(
            tx,
            solana_client::rpc_config::RpcSendTransactionConfig {
                skip_preflight: true,
                max_retries: Some(max_retries as usize),
                preflight_commitment: Some(CommitmentLevel::Processed),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))
}

pub async fn simulate_tx_with_retry(
    tx: &VersionedTransaction,
    _max_retries: u64,
) -> anyhow::Result<SimulationResult> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();
    info!("Transaction size after compilation: {} bytes", tx_size);

    let response = rpc::rpc_client().simulate_transaction(tx).await?;
    Ok(SimulationResult::from(&response.value))
}

pub async fn fetch_tx(signature: &str) -> anyhow::Result<Transaction> {
    rpc::rpc_client()
        .get_transaction_with_config(&signature.to_sig(), rpc::json_config())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))?
        .to_unified()
}

pub fn fetch_tx_sync(client: &RpcClient, signature: &str) -> anyhow::Result<Transaction> {
    client
        .get_transaction_with_config(&signature.to_sig(), rpc::json_config())
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))?
        .to_unified()
}
