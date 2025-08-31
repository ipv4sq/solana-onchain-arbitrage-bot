use crate::arb::dex::interface::PoolDataLoader;
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
use crate::arb::global::enums::direction::Direction;
use crate::arb::pipeline::trade_strategy::token_balance_cache::get_balance_of_account;
use crate::arb::util::alias::{AResult, MintAddress};
use crate::arb::util::traits::option::OptionExt;
use crate::f;
use rust_decimal::Decimal;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

const FEE_RATE_DENOMINATOR: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct CpmmAmmConfig {
    pub discriminator: [u8; 8],
    pub bump: u8,
    pub disable_create_pool: bool,
    pub index: u16,
    pub trade_fee_rate: u64,
    pub protocol_fee_rate: u64,
    pub fund_fee_rate: u64,
    pub create_pool_fee: u64,
    pub protocol_owner: Pubkey,
    pub fund_owner: Pubkey,
    pub creator_fee_rate: u64,
    pub padding: [u64; 15],
}

fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    (numerator + denominator - 1) / denominator
}

fn calculate_trade_fee(amount: u64, trade_fee_rate: u64) -> u64 {
    ceil_div(amount as u128 * trade_fee_rate as u128, FEE_RATE_DENOMINATOR as u128) as u64
}

fn calculate_creator_fee(amount: u64, creator_fee_rate: u64) -> u64 {
    ceil_div(amount as u128 * creator_fee_rate as u128, FEE_RATE_DENOMINATOR as u128) as u64
}

fn swap_base_input_without_fees(
    input_amount: u64,
    input_vault_amount: u64,
    output_vault_amount: u64,
) -> u64 {
    let numerator = (input_amount as u128) * (output_vault_amount as u128);
    let denominator = (input_vault_amount as u128) + (input_amount as u128);
    (numerator / denominator) as u64
}

impl RaydiumCpmmPoolData {
    pub async fn mid_price_for_quick_estimate(
        &self,
        from: &MintAddress,
        to: &MintAddress,
    ) -> AResult<DlmmQuote> {
        let token_0_cached = get_balance_of_account(&self.base_vault(), &self.base_mint())
            .await
            .or_err(f!(
                "Unable to get balance of owner {} mint {}",
                self.base_vault(),
                self.base_mint()
            ))?;

        let token_1_cached = get_balance_of_account(&self.quote_vault(), &self.quote_mint())
            .await
            .or_err(f!(
                "Unable to get balance of owner {} mint {}",
                self.quote_vault(),
                self.quote_mint()
            ))?;

        let token_0_balance = token_0_cached.amount;
        let token_0_decimals = token_0_cached.decimals;

        let token_1_balance = token_1_cached.amount;
        let token_1_decimals = token_1_cached.decimals;

        let token_0_balance_dec =
            Decimal::from(token_0_balance) / Decimal::from(10u64.pow(token_0_decimals as u32));
        let token_1_balance_dec =
            Decimal::from(token_1_balance) / Decimal::from(10u64.pow(token_1_decimals as u32));

        let price_token_0_per_token_1 = token_1_balance_dec / token_0_balance_dec;

        let mid_price = match self.dir(from, to) {
            Direction::XtoY => price_token_0_per_token_1,
            Direction::YtoX => Decimal::ONE / price_token_0_per_token_1,
        };

        Ok(DlmmQuote { mid_price })
    }

    pub async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
        amm_config: &CpmmAmmConfig,
    ) -> AResult<u64> {
        let is_base_to_quote = *from_mint == self.base_mint();
        
        let (input_vault, output_vault) = if is_base_to_quote {
            (self.token_0_vault, self.token_1_vault)
        } else if *from_mint == self.quote_mint() {
            (self.token_1_vault, self.token_0_vault)
        } else {
            return Err(anyhow::anyhow!("Invalid from_mint address"));
        };
        
        let input_balance = get_balance_of_account(&input_vault, from_mint)
            .await
            .or_err(f!("Unable to get balance of input vault"))?;
        
        let output_balance = get_balance_of_account(&output_vault, to_mint)
            .await
            .or_err(f!("Unable to get balance of output vault"))?;
        
        let input_reserve = if is_base_to_quote {
            input_balance.amount - self.protocol_fees_token_0 - self.fund_fees_token_0 - self.creator_fees_token_0
        } else {
            input_balance.amount - self.protocol_fees_token_1 - self.fund_fees_token_1 - self.creator_fees_token_1
        };
        
        let output_reserve = if is_base_to_quote {
            output_balance.amount - self.protocol_fees_token_1 - self.fund_fees_token_1 - self.creator_fees_token_1
        } else {
            output_balance.amount - self.protocol_fees_token_0 - self.fund_fees_token_0 - self.creator_fees_token_0
        };
        
        let is_creator_fee_on_input = self.creator_fee_on == 0;
        
        let trade_fee = calculate_trade_fee(input_amount, amm_config.trade_fee_rate);
        
        let input_amount_less_fees = if is_creator_fee_on_input && self.enable_creator_fee {
            let creator_fee = calculate_creator_fee(input_amount, amm_config.creator_fee_rate);
            input_amount - trade_fee - creator_fee
        } else {
            input_amount - trade_fee
        };
        
        let output_amount_swapped = swap_base_input_without_fees(
            input_amount_less_fees,
            input_reserve,
            output_reserve,
        );
        
        let final_output = if !is_creator_fee_on_input && self.enable_creator_fee {
            let creator_fee = calculate_creator_fee(output_amount_swapped, amm_config.creator_fee_rate);
            output_amount_swapped - creator_fee
        } else {
            output_amount_swapped
        };
        
        Ok(final_output)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arb::dex::interface::PoolDataLoader;
    use crate::arb::global::constant::mint::Mints;
    use crate::arb::global::state::rpc::rpc_client;
    use crate::arb::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_raydium_cpmm_mid_price() {
        crate::arb::global::state::db::must_init_db().await;
        
        let pool_address = "BtGUffMEnxrzdjyC3kKAHjGMpG1UdZiVWXZUaSpUv13C".to_pubkey();
        let wsol = Mints::WSOL;
        let eagle = "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey();

        let account = rpc_client()
            .get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account");

        let pool_data =
            RaydiumCpmmPoolData::load_data(&account.data).expect("Failed to load pool data");

        assert_eq!(pool_data.base_mint(), wsol);
        assert_eq!(pool_data.quote_mint(), eagle);

        let quote_wsol_to_eagle = pool_data
            .mid_price_for_quick_estimate(&wsol, &eagle)
            .await
            .expect("Failed to calculate mid price");

        let quote_eagle_to_wsol = pool_data
            .mid_price_for_quick_estimate(&eagle, &wsol)
            .await
            .expect("Failed to calculate mid price");

        println!("1 WSOL = {} EAGLE", quote_wsol_to_eagle.mid_price);
        println!("1 EAGLE = {} WSOL", quote_eagle_to_wsol.mid_price);
        
        // Verify prices are inverses of each other
        let product = quote_wsol_to_eagle.mid_price * quote_eagle_to_wsol.mid_price;
        assert!((product - Decimal::ONE).abs() < Decimal::new(1, 10)); // Allow small rounding error
    }

    #[tokio::test]
    async fn test_raydium_cpmm_get_amount_out() {
        crate::arb::global::state::db::must_init_db().await;
        
        let pool_address = "BtGUffMEnxrzdjyC3kKAHjGMpG1UdZiVWXZUaSpUv13C".to_pubkey();
        let config_address = "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2".to_pubkey();
        let wsol = Mints::WSOL;
        let eagle = "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey();

        let pool_account = rpc_client()
            .get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account");

        let pool_data =
            RaydiumCpmmPoolData::load_data(&pool_account.data).expect("Failed to load pool data");

        let config_account = rpc_client()
            .get_account(&config_address)
            .await
            .expect("Failed to fetch config account");
        
        let amm_config: CpmmAmmConfig = if config_account.data.len() >= 8 {
            BorshDeserialize::try_from_slice(&config_account.data[8..])
                .expect("Failed to parse AMM config")
        } else {
            panic!("Config account data too short");
        };

        println!("Trade fee rate: {} bps", amm_config.trade_fee_rate as f64 / 10000.0);
        println!("Creator fee rate: {} bps", amm_config.creator_fee_rate as f64 / 10000.0);
        
        let input_amount = 1_000_000_000; // 1 WSOL (9 decimals)
        
        let amount_out = pool_data
            .get_amount_out(input_amount, &wsol, &eagle, &amm_config)
            .await
            .expect("Failed to calculate amount out");
        
        println!("Swapping {} WSOL -> {} EAGLE", 
            input_amount as f64 / 1e9, 
            amount_out as f64 / 1e6);
        
        assert!(amount_out > 0);
        
        let reverse_amount_out = pool_data
            .get_amount_out(1_000_000, &eagle, &wsol, &amm_config)
            .await
            .expect("Failed to calculate reverse amount out");
        
        println!("Swapping {} EAGLE -> {} WSOL",
            1_000_000 as f64 / 1e6,
            reverse_amount_out as f64 / 1e9);
        
        assert!(reverse_amount_out > 0);
    }
}
