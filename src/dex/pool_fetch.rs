use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;


pub trait PoolFetch: Sized {
    fn fetch(
        pool: &Pubkey,
        mint: &Pubkey,
        rpc_client: &RpcClient,
    ) -> Result<Self>;
    
    fn fetch_from_str(
        pool_address: &str,
        mint: &Pubkey,
        rpc_client: &RpcClient,
    ) -> Result<Self> {
        use crate::constants::helpers::ToPubkey;
        let pool_pubkey = pool_address.to_pubkey();
        Self::fetch(&pool_pubkey, mint, rpc_client)
    }
}