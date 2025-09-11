use crate::util::env::holder::ENV_CONFIG;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_client::nonblocking::rpc_client;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

static RPC_HOLDER: Lazy<RwLock<Arc<rpc_client::RpcClient>>> = Lazy::new(|| {
    RwLock::new(Arc::new(rpc_client::RpcClient::new_with_commitment(
        ENV_CONFIG.solana_rpc_url.clone(),
        CommitmentConfig::processed(),
    )))
});

pub(super) fn rpc_client() -> Arc<rpc_client::RpcClient> {
    RPC_HOLDER.read().clone()
}

pub fn json_config() -> RpcTransactionConfig {
    RpcTransactionConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::processed()),
        max_supported_transaction_version: None,
    }
}

#[cfg(test)]
pub fn _set_test_client() {
    *RPC_HOLDER.write() = Arc::new(rpc_client::RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::processed(),
    ));
}
