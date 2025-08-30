use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{InputAccountUtil, PoolDataLoader};
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::pump_amm::input_account::PumpAmmInputAccounts;
use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::arb::dex::refined_interface::{PoolBase, RefinedPoolConfig};
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use crate::arb::util::traits::account_meta::ToAccountMeta;
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

    fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        let built = PumpAmmInputAccounts::build_accounts_no_matter_direction_size(
            payer,
            &self.pool_address,
            &self.pool_data,
        )?;

        let accounts: Vec<AccountMeta> = vec![
            built.program,
            self.pool_data.pair().desired_mint()?.to_readonly(),
            built.global_config,
            built.event_authority,
            built.protocol_fee_recipient,
            built.pool,
            built.pool_base_token_account,
            built.pool_quote_token_account,
            built.protocol_fee_recipient_token_account,
            built.coin_creator_vault_ata,
            built.coin_creator_vault_authority,
            built.global_volume_accumulator.unwrap(),
            built.user_volume_accumulator.unwrap(),
        ];

        Ok(accounts)
    }

    async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote> {
        self.pool_data.mid_price_for_quick_estimate(from, to).await
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
