use crate::dex::interface::PoolDataLoader;
use crate::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
use crate::dex::raydium_cpmm::price::amm_config::CpmmAmmConfig;
use crate::global::state::token_balance_holder::get_balance_of_account;
use crate::util::alias::{AResult, MintAddress};
use crate::util::traits::option::OptionExt;
use crate::f;

pub const FEE_RATE_DENOMINATOR: u64 = 1_000_000;

impl RaydiumCpmmPoolData {
    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        let amm_config = CpmmAmmConfig::get(&self.amm_config).await.or_err("")?;
        self.get_amount_out_with_amm_config(input_amount, from_mint, to_mint, &amm_config)
            .await
    }

    pub async fn get_amount_out_with_amm_config(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
        amm_config: &CpmmAmmConfig,
    ) -> AResult<u64> {
        let is_base_to_quote = *from_mint == self.base_mint();

        let (input_vault, output_vault) = self.get_vault_in_dir(from_mint, to_mint)?;

        let input_balance = get_balance_of_account(&input_vault, from_mint)
            .await
            .or_err(f!("Unable to get balance of input vault"))?;

        let output_balance = get_balance_of_account(&output_vault, to_mint)
            .await
            .or_err(f!("Unable to get balance of output vault"))?;

        let input_reserve = if is_base_to_quote {
            input_balance.amount
                - self.protocol_fees_token_0
                - self.fund_fees_token_0
                - self.creator_fees_token_0
        } else {
            input_balance.amount
                - self.protocol_fees_token_1
                - self.fund_fees_token_1
                - self.creator_fees_token_1
        };

        let output_reserve = if is_base_to_quote {
            output_balance.amount
                - self.protocol_fees_token_1
                - self.fund_fees_token_1
                - self.creator_fees_token_1
        } else {
            output_balance.amount
                - self.protocol_fees_token_0
                - self.fund_fees_token_0
                - self.creator_fees_token_0
        };

        // The creator_fee_on field: 0 means fee on input, 1 means fee on output  
        let is_creator_fee_on_input = self.creator_fee_on == 0;

        let trade_fee = calculate_trade_fee(input_amount, amm_config.trade_fee_rate);

        let input_amount_less_fees = if is_creator_fee_on_input && self.enable_creator_fee {
            let creator_fee = calculate_creator_fee(input_amount, amm_config.creator_fee_rate);
            input_amount - trade_fee - creator_fee
        } else {
            input_amount - trade_fee
        };

        let output_amount_swapped =
            swap_base_input_without_fees(input_amount_less_fees, input_reserve, output_reserve);

        let final_output = if !is_creator_fee_on_input && self.enable_creator_fee {
            let creator_fee =
                calculate_creator_fee(output_amount_swapped, amm_config.creator_fee_rate);
            output_amount_swapped - creator_fee
        } else {
            output_amount_swapped
        };

        Ok(final_output)
    }
}

fn calculate_trade_fee(amount: u64, trade_fee_rate: u64) -> u64 {
    let numerator = (amount as u128) * (trade_fee_rate as u128);
    let denominator = FEE_RATE_DENOMINATOR as u128;
    ((numerator + denominator - 1) / denominator) as u64
}

fn calculate_creator_fee(amount: u64, creator_fee_rate: u64) -> u64 {
    let numerator = (amount as u128) * (creator_fee_rate as u128);
    let denominator = FEE_RATE_DENOMINATOR as u128;
    ((numerator + denominator - 1) / denominator) as u64
}

fn swap_base_input_without_fees(
    input_amount: u64,
    input_vault_amount: u64,
    output_vault_amount: u64,
) -> u64 {
    // Using u128 to prevent overflow
    let input_amount_u128 = input_amount as u128;
    let input_vault_u128 = input_vault_amount as u128;
    let output_vault_u128 = output_vault_amount as u128;
    
    // Standard constant product formula: output = (input * output_reserve) / (input_reserve + input)
    let numerator = input_amount_u128.checked_mul(output_vault_u128).expect("Overflow in numerator");
    let denominator = input_vault_u128.checked_add(input_amount_u128).expect("Overflow in denominator");
    
    // Floor division (matching Solana's behavior)
    (numerator / denominator) as u64
}
