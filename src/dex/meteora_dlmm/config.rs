use crate::convention::chain::instruction::Instruction;
use crate::dex::interface::{PoolBase, PoolConfig, PoolDataLoader};
use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
use crate::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::dex::meteora_dlmm::price::price_calculator::DlmmQuote;
use crate::global::enums::dex_type::DexType;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type MeteoraDlmmConfig = PoolBase<MeteoraDlmmPoolData>;

impl PoolConfig<MeteoraDlmmPoolData> for MeteoraDlmmConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = MeteoraDlmmPoolData::load_data(data)?;
        Ok(MeteoraDlmmConfig {
            pool_address: address,
            base_mint: pool_data.token_x_mint,
            base_reserve: pool_data.reserve_x,
            quote_mint: pool_data.token_y_mint,
            quote_reserve: pool_data.reserve_y,
            dex_type,
            pool_data,
        })
    }

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
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

    async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        self.pool_data
            .get_amount_out(input_amount, from_mint, to_mint, &self.pool_address)
            .await
    }
}

impl AsRef<PoolBase<MeteoraDlmmPoolData>> for MeteoraDlmmConfig {
    fn as_ref(&self) -> &PoolBase<MeteoraDlmmPoolData> {
        self
    }
}
