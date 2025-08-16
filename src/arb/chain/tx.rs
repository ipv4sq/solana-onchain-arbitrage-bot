use crate::arb::constant::client::rpc_client;
use crate::constants::helpers::ToSignature;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

fn json_parsed_config() -> RpcTransactionConfig {
    RpcTransactionConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::JsonParsed),
        commitment: None,
        max_supported_transaction_version: Some(0),
    }
}

pub async fn fetch_tx(signature: &str) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    rpc_client()
        .get_transaction_with_config(&signature.to_sig(), json_parsed_config())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))
}

pub fn fetch_tx_sync(
    client: &RpcClient,
    signature: &str,
) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
    client
        .get_transaction_with_config(&signature.to_sig(), json_parsed_config())
        .map_err(|e| anyhow::anyhow!("Failed to fetch transaction: {}", e))
}
