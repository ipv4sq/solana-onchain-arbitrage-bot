#[allow(non_upper_case_globals)]
#[cfg(test)]
mod tests {
    use crate::dex::interface::PoolConfig;
    use crate::dex::raydium_cpmm::config::RaydiumCpmmConfig;
    use crate::dex::raydium_cpmm::misc::input_account::RaydiumCpmmInputAccount;
    use crate::global::client::db::must_init_db;
    use crate::global::enums::dex_type::DexType;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::account_meta::ToAccountMeta;
    use solana_sdk::pubkey;
    use solana_sdk::pubkey::Pubkey;

    static pool: Pubkey = pubkey!("Q2sPHPdUWFMg7M7wwrQKLrn619cAucfRsmhVJffodSp");
    static payer: Pubkey = pubkey!("Hq8MmCBFavX2GooSCk9XFp4Whue3wmC3jaZqk1zDgSXx");

    #[tokio::test]
    async fn build_command() -> AResult<()> {
        must_init_db().await;

        let config = RaydiumCpmmConfig::from_address(&pool).await?;
        let mut accounts = RaydiumCpmmInputAccount::build_accounts(
            &payer,
            &pool,
            &config.pool_data,
            &config.base_mint,
            &config.quote_mint,
        )
        .await?
        .to_list_cloned();
        accounts.push("4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_readonly());

        // Build the validator command from the accounts array
        let bootstrap_cmd = format!(
            "solana-test-validator --reset \\
  --url https://api.mainnet-beta.solana.com \\
  {}",
            accounts
                .iter()
                .map(|account| {
                    let dex_type = DexType::determine_from(&account.pubkey);
                    return if dex_type != DexType::Unknown {
                        format!("--clone-upgradeable-program {}", account.pubkey)
                    } else {
                        format!("--clone {}", account.pubkey)
                    };
                })
                .collect::<Vec<_>>()
                .join(" \\\n  ")
        );

        println!("\n{}\n", bootstrap_cmd);

        unit_ok!()
    }
}
