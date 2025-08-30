use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{PoolBase, PoolDataLoader, RefinedPoolConfig};
use crate::arb::dex::legacy_interface::InputAccountUtil;
use crate::arb::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
use crate::arb::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use crate::arb::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type MeteoraDlmmRefinedConfig = PoolBase<MeteoraDlmmPoolData>;

impl RefinedPoolConfig<MeteoraDlmmPoolData> for MeteoraDlmmRefinedConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = MeteoraDlmmPoolData::load_data(data)?;
        Ok(MeteoraDlmmRefinedConfig {
            pool_address: address,
            base_mint: pool_data.token_x_mint,
            quote_mint: pool_data.token_y_mint,
            dex_type,
            pool_data,
        })
    }

    fn extract_pool_from(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&DexType::MeteoraDlmm.owner_program_id())?;
        let address = ix.account_at(0)?.pubkey;
        Ok((DexType::MeteoraDlmm, address))
    }

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        let built = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
            payer,
            &self.pool_address,
            &self.pool_data,
        )
        .await?;
        let accounts: Vec<AccountMeta> = [
            vec![
                built.program,
                self.pool_data.pair().desired_mint()?.to_readonly(),
                built.event_authority,
                built.lb_pair,
                built.reverse_x,
                built.reverse_y,
                built.oracle,
            ],
            built.bin_arrays,
        ]
        .concat();
        Ok(accounts)
    }

    async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote> {
        self.pool_data.mid_price_for_quick_estimate(from, to).await
    }
}

impl AsRef<PoolBase<MeteoraDlmmPoolData>> for MeteoraDlmmRefinedConfig {
    fn as_ref(&self) -> &PoolBase<MeteoraDlmmPoolData> {
        self
    }
}
