use once_cell::sync::Lazy;
use parking_lot::RwLock;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;

static RPC_HOLDER: Lazy<RwLock<Arc<RpcClient>>> = Lazy::new(|| {
    let url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    RwLock::new(Arc::new(RpcClient::new(url)))
});

pub fn rpc_client() -> Arc<RpcClient> {
    RPC_HOLDER.read().clone()
}

// 仅测试使用：替换全局句柄
#[cfg(test)]
pub fn _set_test_client(client: RpcClient) {
    *RPC_HOLDER.write() = Arc::new(client);
}
