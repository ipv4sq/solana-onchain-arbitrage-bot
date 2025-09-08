use crate::util::env::holder::ENV_CONFIG;
use once_cell::sync::Lazy;
use solana_client::nonblocking::rpc_client;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

static RPC_HOLDER: Lazy<Arc<rpc_client::RpcClient>> = Lazy::new(|| {
    Arc::new(rpc_client::RpcClient::new_with_commitment(
        ENV_CONFIG.solana_rpc_url.clone(),
        CommitmentConfig::processed(),
    ))
});

pub fn rpc_client() -> Arc<rpc_client::RpcClient> {
    RPC_HOLDER.clone()
}

pub fn json_config() -> RpcTransactionConfig {
    RpcTransactionConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::processed()),
        max_supported_transaction_version: None,
    }
}
