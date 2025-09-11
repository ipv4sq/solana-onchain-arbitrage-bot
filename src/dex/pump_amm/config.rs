use crate::convention::chain::instruction::Instruction;
use crate::dex::interface::{PoolBase, PoolConfig, PoolDataLoader};
use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::meteora_dlmm::price::price_calculator::DlmmQuote;
use crate::dex::pump_amm::misc::input_account::PumpAmmInputAccounts;
use crate::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::dex::pump_amm::PUMP_GLOBAL_CONFIG;
use crate::global::constant::mint::Mints;
use crate::global::enums::dex_type::DexType;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::structs::mint_pair::MintPair;
use crate::util::traits::account_meta::ToAccountMeta;
use crate::util::traits::option::OptionExt;
use chrono::Utc;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;

pub type PumpAmmConfig = PoolBase<PumpAmmPoolData>;

impl PoolConfig<PumpAmmPoolData> for PumpAmmConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = PumpAmmPoolData::load_data(data)?;
        Ok(PumpAmmConfig {
            pool_address: address,
            base_mint: pool_data.base_mint,
            base_reserve: pool_data.pool_base_token_account,
            quote_mint: pool_data.quote_mint,
            quote_reserve: pool_data.pool_quote_token_account,
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
        let fee_program = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
        let pump_program = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
        let pump_amm_program = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

        let (pump_fee_config_pda, _) =
            Pubkey::find_program_address(&[b"fee_config", &pump_program.as_ref()], &fee_program);

        let (pump_amm_fee_config_pda, _) = Pubkey::find_program_address(
            &[b"fee_config", &pump_amm_program.as_ref()],
            &fee_program,
        );

        let mut accounts: Vec<AccountMeta> = vec![
            built.program,
            self.pool_data.mint_pair().desired_mint()?.to_readonly(),
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

        let cutoff_timestamp =
            chrono::DateTime::parse_from_rfc3339("2025-09-01T20:00:00Z")?.timestamp();
        let current_timestamp = Utc::now().timestamp();

        if current_timestamp >= cutoff_timestamp {
            accounts.push(pump_fee_config_pda.to_readonly());
            accounts.push(pump_amm_fee_config_pda.to_readonly());
        }

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
            .get_amount_out(input_amount, from_mint, to_mint)
            .await
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
