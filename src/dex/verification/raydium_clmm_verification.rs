use crate::dex::raydium_cpmm::config::RaydiumCpmmConfig;
use crate::dex::raydium_cpmm::misc::input_account::RaydiumCpmmInputAccount;
use crate::global::client::db::must_init_db;
use crate::global::enums::dex_type::DexType;
use crate::unit_ok;
use crate::util::alias::AResult;
use crate::util::traits::account_meta::ToAccountMeta;

#[cfg(test)]
mod test {
    use crate::global::client::db::must_init_db;
    use crate::util::alias::AResult;

    //   #[tokio::test]
    //   async fn build_command() -> AResult<()> {
    //       must_init_db().await;
    //
    //       let config = RaydiumCpmmConfig::from_address(
    //           &crate::dex::verification::raydium_cpmm_verification::tests::POOL,
    //       )
    //       .await?;
    //       let mut accounts = RaydiumCpmmInputAccount::build_accounts(
    //           &crate::dex::verification::raydium_cpmm_verification::tests::payer,
    //           &crate::dex::verification::raydium_cpmm_verification::tests::POOL,
    //           &config.pool_data,
    //           &config.base_mint,
    //           &config.quote_mint,
    //       )
    //       .await?
    //       .to_list_cloned();
    //       accounts.push("4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_readonly());
    //
    //       // Build the validator command from the accounts array
    //       let bootstrap_cmd = format!(
    //           "solana-test-validator --reset \\
    // --url https://api.mainnet-beta.solana.com \\
    // {}",
    //           accounts
    //               .iter()
    //               .map(|account| {
    //                   let dex_type = DexType::determine_from(&account.pubkey);
    //                   return if dex_type != DexType::Unknown {
    //                       format!("--clone-upgradeable-program {}", account.pubkey)
    //                   } else {
    //                       format!("--clone {}", account.pubkey)
    //                   };
    //               })
    //               .collect::<Vec<_>>()
    //               .join(" \\\n  ")
    //       );
    //
    //       println!("\n{}\n", bootstrap_cmd);
    //
    //       unit_ok!()
    //   }
}
