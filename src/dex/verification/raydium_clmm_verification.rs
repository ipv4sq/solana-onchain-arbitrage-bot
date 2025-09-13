#[cfg(test)]
mod test {
    use crate::dex::interface::PoolConfig;
    use crate::dex::raydium_clmm::config::RaydiumClmmConfig;
    use crate::dex::raydium_clmm::ix_account::RaydiumClmmIxAccount;
    use crate::global::client::db::must_init_db;
    use crate::global::enums::dex_type::DexType;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::account_meta::ToAccountMeta;
    use solana_program::pubkey;
    use solana_sdk::pubkey::Pubkey;

    const POOL: Pubkey = pubkey!("3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv");
    const PAYER: Pubkey = pubkey!("MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa");

    #[tokio::test]
    async fn build_command() -> AResult<()> {
        must_init_db().await;

        let config = RaydiumClmmConfig::from_address(&POOL).await?;
        let mut accounts = RaydiumClmmIxAccount::build_accounts_with_direction(
            &PAYER,
            &POOL,
            &config.pool_data,
            &config.quote_mint,
            &config.base_mint,
        )
        .await?
        .to_list();
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
