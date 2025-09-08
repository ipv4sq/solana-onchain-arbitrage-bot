#[cfg(test)]
mod tests {
    use crate::dex::any_pool_config::AnyPoolConfig;
    use crate::dex::any_pool_config::AnyPoolConfig::MeteoraDlmm;
    use crate::dex::interface::PoolConfig;
    use crate::dex::legacy_interface::InputAccountUtil;
    use crate::dex::meteora_dlmm::config::MeteoraDlmmConfig;
    use crate::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
    use crate::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
    use crate::global::client::db::must_init_db;
    use crate::sdk::solana_rpc::rpc::_set_test_client;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::pubkey::ToPubkey;
    use solana_program::pubkey;
    use solana_sdk::pubkey::Pubkey;

    static POOL: Pubkey = pubkey!("5rCf1DM8LjKTw4YqhnoLcngyZYeNnQqztScTogYHAS6");

    #[tokio::test]
    async fn verify_meteora_dlmm() {
        must_init_db().await;
        // _set_test_client();
    }

    #[tokio::test]
    async fn dump_accounts() -> AResult<()> {
        must_init_db().await;

        let config = MeteoraDlmmConfig::from_address(&POOL).await?;
        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let accounts = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
            &payer,
            &POOL,
            &config.pool_data,
        )
        .await?
        .to_list_cloned();

        // Build the validator command from the accounts array
        let bootstrap_cmd = format!(
            "solana-test-validator --reset \\
  --url https://api.mainnet-beta.solana.com \\
  {}",
            accounts
                .iter()
                .map(|account| format!("--clone {}", account.pubkey))
                .collect::<Vec<_>>()
                .join(" \\\n  ")
        );

        println!("\n{}\n", bootstrap_cmd);

        unit_ok!()
    }
}
