use crate::arb::sdk::solana_rpc::buffered_get_account::buffered_get_account;
use crate::arb::util::alias::AResult;
use solana_client::nonblocking::rpc_client;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;

#[allow(async_fn_in_trait)]
pub trait ExtendRpcClient {
    async fn buffered_get_account(&self, address: &Pubkey) -> AResult<Account>;
    async fn buffered_get_account_data(&self, address: &Pubkey) -> AResult<Vec<u8>>;
}

impl ExtendRpcClient for rpc_client::RpcClient {
    async fn buffered_get_account(&self, address: &Pubkey) -> AResult<Account> {
        buffered_get_account(address).await
    }

    async fn buffered_get_account_data(&self, address: &Pubkey) -> AResult<Vec<u8>> {
        let account = buffered_get_account(address).await?;
        Ok(account.data)
    }
}
