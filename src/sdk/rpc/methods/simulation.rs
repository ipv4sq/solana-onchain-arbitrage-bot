use crate::sdk::rpc::client::rpc_client;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_client::rpc_response::{Response, RpcSimulateTransactionResult};
use solana_sdk::transaction::VersionedTransaction;

pub async fn simulate_transaction_with_config(
    transaction: &VersionedTransaction,
    config: RpcSimulateTransactionConfig,
) -> Result<Response<RpcSimulateTransactionResult>, solana_client::client_error::ClientError> {
    rpc_client()
        .simulate_transaction_with_config(transaction, config)
        .await
}
