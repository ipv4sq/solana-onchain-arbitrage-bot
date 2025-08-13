use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;

pub trait PoolFetch: Sized {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> Result<Self>;
}

// Generic function to fetch any pool that implements PoolFetch
pub fn fetch_pool<T: PoolFetch>(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> Result<T> {
    T::fetch(pool, mint, rpc_client)
}
