#[cfg(test)]
mod tests {
    use crate::arb::constant::dex_type::DexType;
    use crate::arb::constant::mint::Mints;
    use crate::arb::global::rpc::{ensure_mint_account_exists, fetch_tx};
    use crate::arb::pool::register::AnyPoolConfig;
    use crate::arb::program::mev_bot::fire::construct::*;
    use crate::arb::program::mev_bot::ix::extract_mev_instruction;
    use crate::arb::strategy::unmarshal::read_from_database;
    use crate::constants::helpers::ToPubkey;
    use itertools::Itertools;
    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{read_keypair_file, Keypair};
    use std::cmp::min;
    use tracing::info;

    fn get_wallet() -> Keypair {
        let wallet_json_path = "/Users/l/Downloads/test_jz.json";
        let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
        return wallet;
    }

    fn minor_mint() -> Pubkey {
        "9yBQVHj2FJnf7XfQWUPQoj3iyMwAXQMxBWD37cwFBAGS".to_pubkey()
    }

    #[tokio::test]
    async fn create_wsol_ata() {
        // let result = ensure_mint_account_exists(&Mints::WSOL.to_pubkey(), &get_wallet()).await;
        let _ = ensure_mint_account_exists(&minor_mint(), &get_wallet()).await;
    }

    #[tokio::test]
    async fn test_send_tx() {
        let wallet_json_path = "/Users/l/Downloads/test_jz.json";
        let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
        let unit_price = 10_000;
        let compute_unit_limit = 400_000;
        let meteora_dlmm_pool = "3odMjqSfsfj9uGHg7Ax4UWmiayCzQXZn6gNpmuxpttSk".to_pubkey();
        let meteora_damm_v2_pool = "G2TGspLi4G1LfH8ExkiMNS5mCZsgKvKtSBP6rNwMavd9".to_pubkey();
        let result = build_and_send(
            &wallet,
            &minor_mint(),
            compute_unit_limit,
            unit_price,
            vec![
                AnyPoolConfig::from_address(&meteora_dlmm_pool, DexType::MeteoraDlmm)
                    .await
                    .expect("Failed to load pool config"),
                AnyPoolConfig::from_address(&meteora_damm_v2_pool, DexType::MeteoraDammV2)
                    .await
                    .expect("Failed to load pool config"),
            ],
            1000,
        )
        .await;
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn example_tx() {
        let tx = "5xt23Gje9fJJH3EA2etsgfGkkdAKHMaWexeGo6BrzbXRZWVvrxr6erazi4wPSYFq7eyZSvE7kJXF86rTYDDJV4F3";
        let transaction = fetch_tx(&tx).await.unwrap();
        let (ix, _) =
            extract_mev_instruction(&transaction).expect("Failed to extract MEV instruction");
        info!("printing our all the accounts");
        ix.accounts.iter().for_each(|account| {
            println!(
                "account: {}, signer: {}, writable: {}",
                account.pubkey, account.is_signer, account.is_writable
            )
        });
        info!("finished printing our all the accounts");
    }

    #[tokio::test]
    async fn reproduce() {
        let meteora_dlmm_pool = "3odMjqSfsfj9uGHg7Ax4UWmiayCzQXZn6gNpmuxpttSk".to_pubkey();
        let meteora_damm_v2_pool = "G2TGspLi4G1LfH8ExkiMNS5mCZsgKvKtSBP6rNwMavd9".to_pubkey();
        let ix = create_invoke_mev_instruction(
            &"DvLTm5iR43m7u2Rh5rwNmwrKDtD9X8iHpaoLhaUnEKEq".to_pubkey(),
            &minor_mint(),
            1,
            vec![
                AnyPoolConfig::from_address(&meteora_dlmm_pool, DexType::MeteoraDlmm)
                    .await
                    .expect("Failed to load pool config"),
                AnyPoolConfig::from_address(&meteora_damm_v2_pool, DexType::MeteoraDammV2)
                    .await
                    .expect("Failed to load pool config"),
            ],
            1000,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_polling() {
        let pools_data = read_from_database().await.unwrap();
        let wallet = get_wallet();
        let unit_price = 10_000;
        let compute_unit_limit = 400_000;
        let minimum_profit = 1_000_000; // 0.001 SOL minimum profit

        let pools_of_mint = pools_data
            .into_iter()
            .find_or_first(|p| p.pools.len() > 1)
            .expect("No pools found for the target mint");

        // Convert PoolInfo to AnyPoolConfig
        let pool_configs = futures::future::join_all(pools_of_mint.pools.into_iter().map(
            |pool_info| async move {
                AnyPoolConfig::from_address(&pool_info.pool_id, pool_info.dex_type).await
            },
        ))
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

        let result = build_and_send(
            &wallet,
            &pools_of_mint.minor_mint,
            compute_unit_limit,
            unit_price,
            pool_configs,
            minimum_profit,
        )
        .await;

        println!("Result: {:?}", result);
    }
}
