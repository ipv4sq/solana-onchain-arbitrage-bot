#[cfg(test)]
mod test {
    use crate::dex::interface::PoolConfig;
    use crate::dex::raydium_clmm::config::RaydiumClmmConfig;
    use crate::dex::raydium_clmm::ix_account::RaydiumClmmIxAccount;
    use crate::dex::whirlpool::config::WhirlpoolConfig;
    use crate::dex::whirlpool::ix_account::WhirlpoolIxAccount;
    use crate::global::client::db::must_init_db;
    use crate::global::constant::pool_program::PoolProgram;
    use crate::global::enums::dex_type::DexType;
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::account_meta::ToAccountMeta;
    use solana_sdk::pubkey;
    use solana_sdk::pubkey::Pubkey;

    const POOL: Pubkey = pubkey!("HyA4ct7i4XvZsVrLyb5VJhcTP1EZVDZoF9fFGym16zcj");
    const PAYER: Pubkey = pubkey!("BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ");

    #[tokio::test]
    async fn build_command() -> AResult<()> {
        must_init_db().await;

        let config = WhirlpoolConfig::from_address(&POOL).await?;
        let mut accounts = WhirlpoolIxAccount::build_accounts_with_direction(
            &PAYER,
            &POOL,
            &config.pool_data,
            &config.base_mint,
            &config.quote_mint,
        )
        .await?
        .to_list();
        accounts.push("4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_readonly());
        accounts.push(PoolProgram::WHIRLPOOL.to_readonly());

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
