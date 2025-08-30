use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::PoolDataLoader;
use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::arb::dex::refined_interface::{PoolBase, RefinedPoolConfig};
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

type PumpAmmRefinedConfig = PoolBase<PumpAmmPoolData>;

impl RefinedPoolConfig<PumpAmmPoolData> for PumpAmmRefinedConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = PumpAmmPoolData::load_data(data)?;
        Ok(PumpAmmRefinedConfig {
            pool_address: address,
            base_mint: pool_data.base_mint,
            quote_mint: pool_data.quote_mint,
            dex_type,
            pool_data,
        })
    }

    fn extract_pool_from(ix: Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&DexType::PumpAmm.owner_program_id())?;
        let address = ix.account_at(0)?.pubkey;
        Ok((DexType::PumpAmm, address))
    }

    fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> Vec<AccountMeta> {
        todo!()
    }

    fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> u64 {
        todo!()
    }

    fn refresh_pool_data(&mut self, data: &[u8]) -> AResult<&Self> {
        let pool_data = PumpAmmPoolData::load_data(data)?;
        self.pool_data = pool_data;
        Ok(self)
    }
}

impl AsRef<PoolBase<PumpAmmPoolData>> for PoolBase<PumpAmmPoolData> {
    fn as_ref(&self) -> &PoolBase<PumpAmmPoolData> {
        self
    }
}
