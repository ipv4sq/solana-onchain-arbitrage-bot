use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::Transaction;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::util::alias::AResult;
use crate::arb::util::structs::mint_pair::MintPair;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub trait PoolDataLoader: Sized + Serialize + for<'de> Deserialize<'de> {
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

    fn pair(&self) -> MintPair {
        return MintPair(self.base_mint(), self.quote_mint());
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "Data: serde::Serialize + for<'a> serde::Deserialize<'a>")]
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
        let config = Self::from_pool_data(pool, pool_data, pair.desired_mint()?);
        config
    }
}

pub struct TradeDirection {
    pub from: Pubkey,
    pub to: Pubkey,
}

pub trait InputAccountUtil<Account, Data>: Sized {
    fn restore_from(ix: &Instruction, tx: &Transaction) -> Result<Account>;

    /*
    1. This is just for building the right list of accounts with correct permission set.
    2. If there is any bin array, it would quickly estimate.
     */
    fn build_accounts_no_matter_direction_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &Data,
    ) -> Result<Account>;

    // This is the most accurate version, for you to generate swap instructions directly in the future
    fn build_accounts_with_direction_and_size(
        payer: &Pubkey,
        pool: &Pubkey,
        pool_data: &Data,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
    ) -> Result<Account>;

    fn get_trade_direction(self) -> AResult<TradeDirection>;

    fn to_list(&self) -> Vec<&AccountMeta>;

    fn to_list_cloned(&self) -> Vec<AccountMeta> {
        self.to_list().into_iter().cloned().collect()
    }
}

pub trait PriceCalculation {
    fn calculate_price(&self, tx: &Transaction) -> Result<u64>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    XtoY,
    YtoX,
}
