use crate::arb::database::mint_record::repository::MintRecordRepository;
use crate::arb::dex::interface::PoolDataLoader;
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
use crate::arb::global::enums::direction::Direction;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::pipeline::trade_strategy::token_balance_cache::get_balance_of_account;
use crate::arb::util::alias::{AResult, MintAddress};
use crate::arb::util::traits::option::OptionExt;
use crate::f;
use anyhow::anyhow;
use rust_decimal::Decimal;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

impl PumpAmmPoolData {
    pub fn dir(&self, from: &MintAddress, to: &MintAddress) -> Direction {
        if *from == self.base_mint && *to == self.quote_mint {
            return Direction::XtoY;
        } else if *from == self.quote_mint && *to == self.base_mint {
            return Direction::YtoX;
        }
        panic!();
    }

    pub async fn mid_price_for_quick_estimate(
        &self,
        from: &MintAddress,
        to: &MintAddress,
    ) -> AResult<DlmmQuote> {
        let base_cached = get_balance_of_account(&self.base_vault(), &self.base_mint)
            .await
            .or_err(f!(
                "Unable to get balance of owner {} mint {}",
                self.base_vault(),
                self.base_mint
            ))?;
        let quote_cached = get_balance_of_account(&self.quote_vault(), &self.quote_mint)
            .await
            .or_err(f!(
                "Unable to get balance of owner {} mint {}",
                self.quote_vault(),
                self.quote_mint
            ))?;

        let base_balance = base_cached.amount;
        let base_decimals = base_cached.decimals;

        let quote_balance = quote_cached.amount;
        let quote_decimals = quote_cached.decimals;

        let base_balance_dec =
            Decimal::from(base_balance) / Decimal::from(10u64.pow(base_decimals as u32));
        let quote_balance_dec =
            Decimal::from(quote_balance) / Decimal::from(10u64.pow(quote_decimals as u32));

        let price_base_per_quote = quote_balance_dec / base_balance_dec;

        let mid_price = match self.dir(from, to) {
            Direction::XtoY => price_base_per_quote,
            Direction::YtoX => Decimal::ONE / price_base_per_quote,
        };

        Ok(DlmmQuote { mid_price })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::dex::interface::PoolDataLoader;
    use crate::arb::global::constant::mint::Mints;
    use crate::arb::global::state::rpc::rpc_client;
    use crate::arb::util::traits::pubkey::ToPubkey;
    use rust_decimal::Decimal;
    use solana_program::pubkey::Pubkey;

    #[tokio::test]
    async fn test_pump_amm_price_calculation() {
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

        let quote_cope_to_wsol = pool_data
            .mid_price_for_quick_estimate(&cope, &wsol)
            .await
            .expect("Failed to calculate mid price");

        let quote_wsol_to_cope = pool_data
            .mid_price_for_quick_estimate(&wsol, &cope)
            .await
            .expect("Failed to calculate mid price");

        println!("1 SOL = {} COPE", quote_wsol_to_cope.mid_price);
    }
}
