use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_client::nonblocking::rpc_client;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

static RPC_HOLDER: Lazy<RwLock<Arc<rpc_client::RpcClient>>> = Lazy::new(|| {
    let url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    RwLock::new(Arc::new(rpc_client::RpcClient::new_with_commitment(
        url,
        CommitmentConfig::processed(),
    )))
});

pub fn rpc_client() -> Arc<rpc_client::RpcClient> {
    RPC_HOLDER.read().clone()
}

pub fn json_config() -> RpcTransactionConfig {
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
