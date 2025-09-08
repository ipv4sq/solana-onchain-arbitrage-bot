use crate::dex::interface::PoolDataLoader;
use crate::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::dex::pump_amm::price::global_config::{compute_fees_bps, GlobalConfig};
use crate::global::constant::pool_program::PoolProgram;
use crate::global::state::token_balance_holder::get_balance_of_account;
use crate::util::alias::{AResult, MintAddress};
use crate::util::traits::option::OptionExt;
use crate::f;
use solana_program::pubkey::Pubkey;

pub const FEE_RATE_DENOMINATOR: u64 = 10_000;

impl PumpAmmPoolData {
    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        self.consists_of(from_mint, to_mint)?;

        let (input_vault, output_vault) = self.get_vault_in_dir(from_mint, to_mint)?;

        let input_balance = get_balance_of_account(&input_vault, from_mint)
            .await
            .or_err(f!("Unable to get balance of input vault"))?;

        let output_balance = get_balance_of_account(&output_vault, to_mint)
            .await
            .or_err(f!("Unable to get balance of output vault"))?;

        let input_reserve = input_balance.amount;
        let output_reserve = output_balance.amount;

        let raw_output_amount =
            swap_base_input_without_fees(input_amount, input_reserve, output_reserve);

        let global_config = GlobalConfig::get().await?;

        let fees = compute_fees_bps(
            &global_config,
            None,
            is_pump_pool(&self.base_mint, &self.creator),
            0,
        );

        let lp_fee = calculate_fee(raw_output_amount, fees.lp_fee_bps);
        let protocol_fee = calculate_fee(raw_output_amount, fees.protocol_fee_bps);

        let coin_creator_fee = if self.coin_creator == Pubkey::default() {
            0
        } else {
            calculate_fee(raw_output_amount, fees.creator_fee_bps)
        };

        let final_output = raw_output_amount
            .saturating_sub(lp_fee)
            .saturating_sub(protocol_fee)
            .saturating_sub(coin_creator_fee);

        Ok(final_output)
    }
}

fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    (numerator + denominator - 1) / denominator
}

fn calculate_fee(amount: u64, fee_bps: u64) -> u64 {
    ceil_div(
        amount as u128 * fee_bps as u128,
        FEE_RATE_DENOMINATOR as u128,
    ) as u64
}

fn swap_base_input_without_fees(input_amount: u64, input_reserve: u64, output_reserve: u64) -> u64 {
    if input_reserve == 0 || output_reserve == 0 {
        return 0;
    }

    let numerator = (output_reserve as u128) * (input_amount as u128);
    let denominator = (input_reserve as u128) + (input_amount as u128);
    (numerator / denominator) as u64
}

fn is_pump_pool(base_mint: &Pubkey, pool_creator: &Pubkey) -> bool {
    let (pump_pool_authority, _) = Pubkey::find_program_address(
        &[b"pool_authority", base_mint.as_ref()],
        &PoolProgram::PUMP_AMM,
    );
    pump_pool_authority == *pool_creator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global::client::db::must_init_db;
    use crate::global::constant::mint::Mints;
    use crate::sdk::solana_rpc::rpc::rpc_client;
    use crate::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_pump_amm_get_amount_out() {
        must_init_db().await;

        let pool_address: Pubkey = "URqx24yyYxtXXhTbBQnbtPLhtLWYoaDaRxuQuLpNS3S".to_pubkey();
        let cope: Pubkey = "DMwbVy48dWVKGe9z1pcVnwF3HLMLrqWdDLfbvx8RchhK".to_pubkey();
        let wsol: Pubkey = Mints::WSOL;

        let account = rpc_client()
            .get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account");

        let pool_data =
            PumpAmmPoolData::load_data(&account.data).expect("Failed to load pool data");

        assert_eq!(pool_data.base_mint, cope);
        assert_eq!(pool_data.quote_mint, wsol);

        let input_amount = 1_000_000_000; // 1 COPE

        let amount_out = pool_data
            .get_amount_out(input_amount, &cope, &wsol)
            .await
            .expect("Failed to calculate amount out");

        println!(
            "Swapping {} COPE -> {} WSOL (with fees)",
            input_amount as f64 / 1e9,
            amount_out as f64 / 1e9
        );

        assert!(amount_out > 0);

        let amount_out_reverse = pool_data
            .get_amount_out(amount_out, &wsol, &cope)
            .await
            .expect("Failed to calculate reverse amount out");

        println!(
            "Swapping {} WSOL -> {} COPE (reverse, with fees)",
            amount_out as f64 / 1e9,
            amount_out_reverse as f64 / 1e9
        );

        assert!(amount_out_reverse < input_amount);
    }

    #[test]
    fn test_fee_calculations() {
        assert_eq!(calculate_fee(1000, 100), 10);

        assert_eq!(calculate_fee(1000, 25), 3);

        assert_eq!(calculate_fee(10000, 300), 300);

        assert_eq!(calculate_fee(999, 100), 10);
    }

    #[test]
    fn test_swap_formula() {
        let output = swap_base_input_without_fees(1000, 10000, 10000);
        assert_eq!(output, 909);

        let output = swap_base_input_without_fees(100, 1000000, 1000000);
        assert_eq!(output, 99);

        let output = swap_base_input_without_fees(0, 10000, 10000);
        assert_eq!(output, 0);

        let output = swap_base_input_without_fees(1000, 0, 10000);
        assert_eq!(output, 0);
    }
}
