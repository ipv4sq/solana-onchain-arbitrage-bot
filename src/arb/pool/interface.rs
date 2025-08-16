use crate::arb::constant::mint::MintPair;
use crate::arb::global::rpc::rpc_client;
use anyhow::Result;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiPartiallyDecodedInstruction,
};

pub trait PoolDataLoader: Sized {
    fn load_data(data: &[u8]) -> Result<Self>;

    // mints
    fn base_mint(&self) -> Pubkey;
    fn quote_mint(&self) -> Pubkey;

    // vaults
    fn base_vault(&self) -> Pubkey;
    fn quote_vault(&self) -> Pubkey;

    //
    fn consists_of(&self, mint1: &Pubkey, mint2: &Pubkey) -> Result<()> {
        MintPair(self.base_mint(), self.quote_mint()).consists_of(mint1, mint2)
    }

    fn shall_contain(&self, mint: &Pubkey) -> Result<()> {
        MintPair(self.base_mint(), self.quote_mint()).shall_contain(mint)
    }

    fn the_other_mint(&self, excluded_mint: &Pubkey) -> Result<Pubkey> {
        MintPair(self.base_mint(), self.quote_mint()).the_other_mint()
    }
}

#[derive(Debug, Clone)]
pub struct PoolConfig<Data: PoolDataLoader> {
    pub pool: Pubkey,
    pub data: Data,
    pub desired_mint: Pubkey,
    pub minor_mint: Pubkey,
}

pub trait PoolConfigInit<Data: PoolDataLoader>: Sized {
    fn from_pool_data(pool: &Pubkey, pool_data: Data, desired_mint: Pubkey) -> Result<Self>;

    async fn from_address(pool: &Pubkey) -> Result<Self> {
        let client = rpc_client();
        let data = client.get_account_data(pool).await?;
        let pool_data = Data::load_data(&data)?;
        let pair = MintPair(pool_data.base_mint(), pool_data.quote_mint());
        let config = Self::from_pool_data(pool, pool_data, pair.the_other_mint()?);
        config
    }
}

pub trait InputAccountUtil<Account, Data>: Sized {
    fn restore_from(
        ix: &UiPartiallyDecodedInstruction,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<Account>;

    fn build_accounts(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: Data,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> Result<Account>;

    fn to_list(&self) -> Vec<&AccountMeta>;

    fn to_list_cloned(&self) -> Vec<AccountMeta> {
        self.to_list().into_iter().cloned().collect()
    }
}
