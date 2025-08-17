#[cfg(test)]
mod tests {
    use crate::arb::chain::types::LitePool;
    use crate::arb::constant::dex_type::DexType;
    use crate::arb::constant::mint::{MintPair, USDC_KEY, WSOL_KEY};
    use crate::arb::global::db::get_database;
    use crate::arb::program::mev_bot::onchain_monitor::entry::record_pool_and_mints;
    use crate::constants::helpers::ToPubkey;

    #[tokio::test]
    async fn test_record_with_wsol() {
        let pool = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_pubkey();
        let wsol = *WSOL_KEY;
        let other_mint = "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_pubkey();
        let mints = MintPair(wsol, other_mint);

        let lite_pool = LitePool {
            dex_type: DexType::RaydiumV4,
            pool_address: pool,
            mints: mints.clone(),
        };
        let result = record_pool_and_mints(&lite_pool).await;

        assert!(result.is_ok());

        // Verify the record was saved with WSOL as desired_mint
        let db = get_database().await.unwrap();
        let pools = db.list_pool_mints().await.unwrap();
        let found = pools.iter().find(|p| p.pool_id == pool.to_string());
        assert!(found.is_some());
        assert_eq!(found.unwrap().desired_mint, wsol.to_string());
        assert_eq!(found.unwrap().the_other_mint, other_mint.to_string());
    }

    #[tokio::test]
    async fn test_record_with_usdc() {
        let pool = "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX".to_pubkey();
        let usdc = *USDC_KEY;
        let other_mint = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_pubkey();
        let mints = MintPair(other_mint, usdc); // USDC is second

        let lite_pool = LitePool {
            dex_type: DexType::MeteoraDlmm,
            pool_address: pool,
            mints: mints.clone(),
        };
        let result = record_pool_and_mints(&lite_pool).await;

        assert!(result.is_ok());

        // Verify the record was saved with USDC as desired_mint
        let db = get_database().await.unwrap();
        let pools = db.list_pool_mints().await.unwrap();
        let found = pools.iter().find(|p| p.pool_id == pool.to_string());
        assert!(found.is_some());
        assert_eq!(found.unwrap().desired_mint, usdc.to_string());
        assert_eq!(found.unwrap().the_other_mint, other_mint.to_string());
    }

    #[tokio::test]
    async fn test_skip_without_wsol_or_usdc() {
        let pool = "8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj".to_pubkey();
        let mint1 = "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_pubkey();
        let mint2 = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_pubkey();
        let mints = MintPair(mint1, mint2);

        let lite_pool = LitePool {
            dex_type: DexType::OrcaWhirlpool,
            pool_address: pool,
            mints: mints.clone(),
        };
        let result = record_pool_and_mints(&lite_pool).await;

        assert!(result.is_ok());

        // Verify the record was NOT saved
        let db = get_database().await.unwrap();
        let pools = db.list_pool_mints().await.unwrap();
        let found = pools.iter().find(|p| p.pool_id == pool.to_string());
        assert!(found.is_none());
    }
}
