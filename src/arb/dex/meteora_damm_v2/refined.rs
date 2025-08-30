use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{PoolBase, PoolDataLoader, RefinedPoolConfig};
use crate::arb::dex::legacy_interface::InputAccountUtil;
use crate::arb::dex::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::global::enums::dex_type::DexType::MeteoraDammV2;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use crate::arb::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type MeteoraDammV2RefinedConfig = PoolBase<MeteoraDammV2PoolData>;

impl RefinedPoolConfig<MeteoraDammV2PoolData> for MeteoraDammV2RefinedConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = MeteoraDammV2PoolData::load_data(data)?;
        Ok(MeteoraDammV2RefinedConfig {
            pool_address: address,
            base_mint: pool_data.token_a_mint,
            quote_mint: pool_data.token_b_mint,
            dex_type,
            pool_data,
        })
    }

    fn extract_pool_from(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&MeteoraDammV2.owner_program_id())?;
        let address = ix.account_at(1)?.pubkey;
        Ok((MeteoraDammV2, address))
    }

    fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        let built = MeteoraDammV2InputAccount::build_accounts_no_matter_direction_size(
            payer,
            &self.pool_address,
            &self.pool_data,
        )?;
        let accounts: Vec<AccountMeta> = vec![
            built.meteora_program,
            self.pool_data.pair().desired_mint()?.to_readonly(),
            built.event_authority,
            built.pool_authority,
            self.pool_address.to_writable(),
            built.token_a_vault,
            built.token_b_vault,
        ];
        Ok(accounts)
    }

    async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote> {
        self.pool_data.mid_price_for_quick_estimate(from, to).await
    }
}

impl AsRef<PoolBase<MeteoraDammV2PoolData>> for MeteoraDammV2RefinedConfig {
    fn as_ref(&self) -> &PoolBase<MeteoraDammV2PoolData> {
        self
    }
}
