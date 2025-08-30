use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{Direction, PoolDataLoader};
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use borsh::schema::add_definition;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub struct PoolBase<Data: PoolDataLoader> {
    pub pool_address: PoolAddress,
    pub base_mint: MintAddress,
    pub quote_mint: MintAddress,
    pub dex_type: DexType,
    pub pool_data: Data,
}

#[allow(async_fn_in_trait)]
pub trait RefinedPoolConfig<Data: PoolDataLoader>: AsRef<PoolBase<Data>> {
    async fn from_address(address: &PoolAddress) -> AResult<Self>
    where
        Self: Sized,
    {
        let account = rpc_client().get_account(address).await?;
        let dex_type = DexType::determine_from(&account.owner);
        Self::from_data(*address, dex_type, &account.data)
    }

    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self>
    where
        Self: Sized;

    fn extract_pool_from(ix: Instruction) -> AResult<(DexType, PoolAddress)>;

    fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>>;

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

    // not sure if needed in the future
    fn refresh_pool_data(&mut self, data: &[u8]) -> AResult<Self>
    where
        Self: Sized,
    {
        let x = self.as_ref();
        Self::from_data(x.pool_address, x.dex_type, data)
    }
}
