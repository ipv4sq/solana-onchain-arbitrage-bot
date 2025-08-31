use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{PoolBase, PoolDataLoader, RefinedPoolConfig};
use crate::arb::dex::legacy_interface::InputAccountUtil;
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::pump_amm::misc::input_account::PumpAmmInputAccounts;
use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::arb::dex::pump_amm::PUMP_GLOBAL_CONFIG;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;
use crate::arb::util::traits::account_meta::ToAccountMeta;
use crate::arb::util::traits::option::OptionExt;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type PumpAmmRefinedConfig = PoolBase<PumpAmmPoolData>;

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

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&DexType::PumpAmm.owner_program_id())?;
        let address = find_pump_swap(ix).or_err("Can not find pump swap address")?;
        Ok((DexType::PumpAmm, address))
    }

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        let built = PumpAmmInputAccounts::build_accounts_no_matter_direction_size(
            payer,
            &self.pool_address,
            &self.pool_data,
        )
        .await?;

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
}

impl AsRef<PoolBase<PumpAmmPoolData>> for PoolBase<PumpAmmPoolData> {
    fn as_ref(&self) -> &PoolBase<PumpAmmPoolData> {
        self
    }
}

pub fn find_pump_swap(ix: &Instruction) -> Option<PoolAddress> {
    /*
    #1 - Pool:Pump.fun AMM ( USDC-WSOL) Market
    #2 - User:
    #3 - Global Config:
    #4 - Base Mint:
    #5 - Quote Mint:
    */
    if ix.accounts.len() < 6 {
        return None;
    }
    let account_1 = ix.accounts.get(0)?.pubkey;
    let account_3 = ix.accounts.get(2)?.pubkey;
    let account_4 = ix.accounts.get(3)?.pubkey;
    let account_5 = ix.accounts.get(4)?.pubkey;

    if account_3 != PUMP_GLOBAL_CONFIG {
        return None;
    }

    let pair = MintPair(account_4, account_5);
    if !pair.contains(&Mints::WSOL) || !pair.contains(&Mints::USDC) {
        return None;
    }

    Some(account_1)
}
