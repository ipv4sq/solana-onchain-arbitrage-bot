use crate::convention::chain::instruction::Instruction;
use crate::dex::meteora_dlmm::price::price_calculator::DlmmQuote;
use crate::global::enums::dex_type::DexType;
use crate::global::enums::direction::Direction;
use crate::sdk::solana_rpc::methods::account::buffered_get_account;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::structs::mint_pair::MintPair;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "Data: serde::Serialize + for<'a> serde::Deserialize<'a>")]
pub struct PoolBase<Data: PoolDataLoader> {
    pub pool_address: PoolAddress,
    pub base_mint: MintAddress,
    pub quote_mint: MintAddress,
    pub base_reserve: Pubkey,
    pub quote_reserve: Pubkey,
    pub dex_type: DexType,
    pub pool_data: Data,
}

#[allow(async_fn_in_trait)]
pub trait PoolConfig<Data: PoolDataLoader>: AsRef<PoolBase<Data>> {
    async fn from_address(address: &PoolAddress) -> AResult<Self>
    where
        Self: Sized,
    {
        let account = buffered_get_account(address).await?;
        let dex_type = DexType::determine_from(&account.owner);
        Self::from_data(*address, dex_type, &account.data)
    }

    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self>
    where
        Self: Sized;

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)>;

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>>;

    fn dir(&self, from: &MintAddress, to: &MintAddress) -> Direction {
        let pool_base = self.as_ref();
        if *from == pool_base.base_mint && *to == pool_base.quote_mint {
            return Direction::XtoY;
        } else if *from == pool_base.quote_mint && *to == pool_base.base_mint {
            return Direction::YtoX;
        }
        panic!("pool doesn't contain from and to");
    }

    async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote>;

    async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64>;

    // not sure if needed in the future
    fn refresh_pool_data(&mut self, data: &[u8]) -> AResult<Self>
    where
        Self: Sized,
    {
        let x = self.as_ref();
        Self::from_data(x.pool_address, x.dex_type, data)
    }

    // proxy fields for enum
    fn pool_address(&self) -> PoolAddress {
        self.as_ref().pool_address
    }
    fn base_mint(&self) -> MintAddress {
        self.as_ref().base_mint
    }
    fn quote_mint(&self) -> MintAddress {
        self.as_ref().quote_mint
    }
    fn dex_type(&self) -> DexType {
        self.as_ref().dex_type
    }
    fn base_reserve(&self) -> Pubkey {
        self.as_ref().base_reserve
    }
    fn quote_reserve(&self) -> Pubkey {
        self.as_ref().quote_reserve
    }

    // derived
    fn mint_pair(&self) -> MintPair {
        self.as_ref().pool_data.mint_pair()
    }
    fn pool_data_json(&self) -> Value {
        json!(self.as_ref().pool_data)
    }
}

pub trait PoolDataLoader: Sized + Serialize + for<'de> Deserialize<'de> {
    fn load_data(data: &[u8]) -> anyhow::Result<Self>;

    // mints
    fn base_mint(&self) -> Pubkey;
    fn quote_mint(&self) -> Pubkey;

    // vaults
    fn base_vault(&self) -> Pubkey;
    fn quote_vault(&self) -> Pubkey;

    //
    fn consists_of(&self, mint1: &Pubkey, mint2: &Pubkey) -> anyhow::Result<()> {
        MintPair(self.base_mint(), self.quote_mint()).consists_of(mint1, mint2)
    }

    fn shall_contain(&self, mint: &Pubkey) -> anyhow::Result<()> {
        MintPair(self.base_mint(), self.quote_mint()).shall_contain(mint)
    }

    fn mint_pair(&self) -> MintPair {
        MintPair(self.base_mint(), self.quote_mint())
    }

    fn dir(&self, from: &MintAddress, to: &MintAddress) -> Direction {
        if *from == self.base_mint() && *to == self.quote_mint() {
            return Direction::XtoY;
        } else if *from == self.quote_mint() && *to == self.base_mint() {
            return Direction::YtoX;
        }
        panic!();
    }

    fn get_vault_in_dir(&self, from: &MintAddress, to: &MintAddress) -> AResult<(Pubkey, Pubkey)> {
        self.consists_of(from, to)?;
        if self.base_mint() == *from {
            Ok((self.base_vault(), self.quote_vault()))
        } else {
            Ok((self.quote_vault(), self.base_vault()))
        }
    }
}
