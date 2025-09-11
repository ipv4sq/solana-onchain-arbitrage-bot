use crate::convention::chain::util::simulation::SimulationResult;
use crate::sdk::solana_rpc::client;
use crate::sdk::solana_rpc::client::rpc_client;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_client::rpc_response::{Response, RpcSimulateTransactionResult};
use solana_sdk::transaction::VersionedTransaction;
use tracing::info;

pub async fn simulate_tx_with_retry(
    tx: &VersionedTransaction,
    _max_retries: u64,
) -> anyhow::Result<SimulationResult> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();
    info!("Transaction size after compilation: {} bytes", tx_size);

    let response = client::rpc_client().simulate_transaction(tx).await?;
    Ok(SimulationResult::from(&response.value))
}

pub async fn simulate_transaction_with_config(
    transaction: &VersionedTransaction,
    config: RpcSimulateTransactionConfig,
) -> Result<Response<RpcSimulateTransactionResult>, solana_client::client_error::ClientError> {
    rpc_client()
        .simulate_transaction_with_config(transaction, config)
        .await
}
