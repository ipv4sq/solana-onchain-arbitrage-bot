use crate::dex::interface::PoolDataLoader;
use crate::dex::meteora_dlmm::price::price_calculator::DlmmQuote;
use crate::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
use crate::f;
use crate::global::enums::direction::Direction;
use crate::global::state::token_balance_holder::get_balance_of_account;
use crate::util::alias::{AResult, MintAddress};
use crate::util::traits::option::OptionExt;
use rust_decimal::Decimal;

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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dex::interface::PoolDataLoader;
    use crate::global::client::db::must_init_db;
    use crate::global::constant::mint::Mints;
    use crate::sdk::solana_rpc::methods::account::buffered_get_account;
    use crate::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_raydium_cpmm_mid_price() {
        must_init_db().await;

        let pool_address = "BtGUffMEnxrzdjyC3kKAHjGMpG1UdZiVWXZUaSpUv13C".to_pubkey();
        let wsol = Mints::WSOL;
        let eagle = "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey();

        let account = buffered_get_account(&pool_address)
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
        must_init_db().await;

        let pool_address = "BtGUffMEnxrzdjyC3kKAHjGMpG1UdZiVWXZUaSpUv13C".to_pubkey();
        let wsol = Mints::WSOL;
        let eagle = "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey();

        let pool_account = buffered_get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account");

        let pool_data =
            RaydiumCpmmPoolData::load_data(&pool_account.data).expect("Failed to load pool data");

        let input_amount = 1_000_000_000; // 1 WSOL

        let amount_out = pool_data
            .get_amount_out(input_amount, &wsol, &eagle)
            .await
            .expect("Failed to calculate amount out");

        println!(
            "Swapping {} WSOL -> {} EAGLE (auto-fetched config)",
            input_amount as f64 / 1e9,
            amount_out as f64 / 1e6
        );

        assert!(amount_out > 0);
    }
}
