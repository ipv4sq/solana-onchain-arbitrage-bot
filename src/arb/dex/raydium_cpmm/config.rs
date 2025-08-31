use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{PoolBase, PoolConfig};
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::raydium_cpmm::pool_data::RaydiumCpmmAPoolData;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type RaydiumCpmmConfig = PoolBase<RaydiumCpmmAPoolData>;

impl PoolConfig<RaydiumCpmmAPoolData> for RaydiumCpmmConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        todo!()
    }

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        todo!()
    }

    async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote> {
        todo!()
    }
}

impl AsRef<PoolBase<RaydiumCpmmAPoolData>> for RaydiumCpmmConfig {
    fn as_ref(&self) -> &PoolBase<RaydiumCpmmAPoolData> {
        self
    }
}
