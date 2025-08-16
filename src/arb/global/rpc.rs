use std::sync::Arc;
use crate::constants::helpers::ToSignature;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_client::nonblocking::rpc_client;

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

static RPC_HOLDER: Lazy<RwLock<Arc<rpc_client::RpcClient>>> = Lazy::new(|| {
    let url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    RwLock::new(Arc::new(rpc_client::RpcClient::new(url)))
});

pub fn rpc_client() -> Arc<rpc_client::RpcClient> {
    RPC_HOLDER.read().clone()
}

// 仅测试使用：替换全局句柄
#[cfg(test)]
pub fn _set_test_client(client: rpc_client::RpcClient) {
    *RPC_HOLDER.write() = Arc::new(client);
}