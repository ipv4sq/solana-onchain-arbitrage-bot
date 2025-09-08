use crate::convention::chain::util::alt::get_alt_by_key;
use crate::dex::meteora_dlmm::misc::input_data::MeteoraDlmmIxData;
use crate::global::constant::pool_program::PoolProgram;
use crate::global::daemon::blockhash::get_blockhash;
use crate::global::enums::step_type::StepType;
use crate::pipeline::uploader::mev_bot::construct::gas_instructions;
use crate::sdk::solana_rpc::rpc::rpc_client;
use crate::util::alias::AResult;
use crate::util::traits::pubkey::ToPubkey;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::message::v0::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

async fn build_tx(signer: Pubkey, accounts: Vec<AccountMeta>) -> AResult<VersionedTransaction> {
    let (mut instructions, _limit) = gas_instructions(100_000, 0);
    let data = MeteoraDlmmIxData {
        amount_in: 10000000000,
        min_amount_out: 0,
    };
    let swap_ix = Instruction {
        program_id: PoolProgram::METEORA_DLMM,
        accounts: accounts.clone(),
        data: hex::decode(data.to_hex())?,
    };
    instructions.push(swap_ix);
    let alt_keys = vec![
        // this seems to be legit
        "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
    ];

    let mut alts = Vec::new();
    for key in &alt_keys {
        alts.push(get_alt_by_key(key).await?);
    }
    let blockhash = rpc_client().get_latest_blockhash().await?;

    let message = Message::try_compile(&signer, &instructions, &alts, blockhash)?;

    let tx = VersionedTransaction {
        signatures: vec![Signature::default(); 1],
        message: solana_sdk::message::VersionedMessage::V0(message),
    };
    Ok(tx)
}

#[cfg(test)]
mod tests {
    use crate::dex::interface::PoolConfig;
    use crate::dex::legacy_interface::InputAccountUtil;
    use crate::dex::meteora_dlmm::config::MeteoraDlmmConfig;
    use crate::dex::meteora_dlmm::misc::input_account::MeteoraDlmmInputAccounts;
    use crate::dex::meteora_dlmm::verification::build_tx;
    use crate::global::client::db::must_init_db;
    use crate::sdk::solana_rpc::rpc::{_set_test_client, rpc_client};
    use crate::unit_ok;
    use crate::util::alias::AResult;
    use crate::util::traits::pubkey::ToPubkey;
    use solana_account_decoder::UiAccountEncoding;
    use solana_client::rpc_config::{
        RpcSimulateTransactionAccountsConfig, RpcSimulateTransactionConfig,
    };
    use solana_program::program_pack::Pack;
    use solana_program::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use solana_transaction_status::UiTransactionEncoding;
    use sqlx::__rt::sleep;
    use std::hash::Hash;
    use std::time::Duration;

    static POOL: Pubkey = pubkey!("5rCf1DM8LjKTw4YqhnoLcngyZYeNnQqztScTogYHAS6");

    #[tokio::test]
    async fn verify_meteora_dlmm() {
        must_init_db().await;
        // _set_test_client();
    }

    #[tokio::test]
    async fn simulate() -> AResult<()> {
        _set_test_client();
        must_init_db().await;
        
        // Give services time to initialize
        sleep(Duration::from_millis(100)).await;

        let config = MeteoraDlmmConfig::from_address(&POOL).await?;
        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let accounts = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
            &payer,
            &POOL,
            &config.pool_data,
        )
        .await?
        .to_list_cloned();
        let tx = build_tx(payer, accounts.clone()).await?;

        // Get the user token accounts from the instruction accounts
        let user_token_in = accounts[4].pubkey;
        let user_token_out = accounts[5].pubkey;

        // Get token mint addresses and symbols
        let token_x_mint = config.pool_data.token_x_mint;
        let token_y_mint = config.pool_data.token_y_mint;

        // Fetch token symbols from database
        use crate::database::mint_record::repository::MintRecordRepository;
        let token_x_record = MintRecordRepository::get_mint(&token_x_mint).await?;
        let token_x_symbol = token_x_record
            .map(|m| m.symbol)
            .unwrap_or_else(|| token_x_mint.to_string()[..6].to_string());

        let token_y_record = MintRecordRepository::get_mint(&token_y_mint).await?;
        let token_y_symbol = token_y_record
            .map(|m| m.symbol)
            .unwrap_or_else(|| token_y_mint.to_string()[..6].to_string());

        println!("Pool: {}", POOL);
        println!("Swap: {} -> {}", token_x_symbol, token_y_symbol);

        // Fetch pre-simulation token balances
        use solana_account_decoder::parse_token::parse_token;
        use spl_token::state::Account as TokenAccount;

        let pre_token_in = rpc_client().get_account(&user_token_in).await?;
        let pre_token_out = rpc_client().get_account(&user_token_out).await?;

        let pre_balance_in = if pre_token_in.lamports > 0 {
            TokenAccount::unpack(&pre_token_in.data)?.amount
        } else {
            0
        };

        let pre_balance_out = if pre_token_out.lamports > 0 {
            TokenAccount::unpack(&pre_token_out.data)?.amount
        } else {
            0
        };

        println!("\nPre-simulation balances:");
        println!("  {} (in):  {}", token_x_symbol, pre_balance_in);
        println!("  {} (out): {}", token_y_symbol, pre_balance_out);

        let response = rpc_client()
            .simulate_transaction_with_config(
                &tx,
                RpcSimulateTransactionConfig {
                    sig_verify: false,
                    replace_recent_blockhash: true,
                    commitment: None,
                    encoding: Some(UiTransactionEncoding::Base64),
                    accounts: Some(RpcSimulateTransactionAccountsConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        addresses: vec![user_token_in.to_string(), user_token_out.to_string()],
                    }),
                    min_context_slot: None,
                    inner_instructions: true,
                },
            )
            .await?;

        println!("\nUnits consumed: {:?}", response.value.units_consumed);

        // Check simulation result
        if let Some(err) = response.value.err {
            println!("Simulation error: {:?}", err);
        } else {
            println!("Simulation successful!");

            // Get post-simulation account states
            if let Some(accounts) = response.value.accounts {
                if accounts.len() >= 2 {
                    // Decode the base64 account data to get actual balances
                    use base64::Engine;

                    if let Some(account_in) = &accounts[0] {
                        if let solana_account_decoder::UiAccountData::Binary(base64_str, _) =
                            &account_in.data
                        {
                            let decoded =
                                base64::engine::general_purpose::STANDARD.decode(base64_str)?;
                            if decoded.len() >= 72 {
                                let post_balance_in = TokenAccount::unpack(&decoded)?.amount;
                                println!("\nPost-simulation balances:");
                                println!(
                                    "  {} (in):  {} (diff: {})",
                                    &token_x_symbol,
                                    post_balance_in,
                                    post_balance_in as i128 - pre_balance_in as i128
                                );
                            }
                        }
                    }

                    if let Some(account_out) = &accounts[1] {
                        if let solana_account_decoder::UiAccountData::Binary(base64_str, _) =
                            &account_out.data
                        {
                            let decoded =
                                base64::engine::general_purpose::STANDARD.decode(base64_str)?;
                            if decoded.len() >= 72 {
                                let post_balance_out = TokenAccount::unpack(&decoded)?.amount;
                                println!(
                                    "  {} (out): {} (diff: +{})",
                                    &token_y_symbol,
                                    post_balance_out,
                                    post_balance_out as i128 - pre_balance_out as i128
                                );
                            }
                        }
                    }
                }
            }

            // Show logs if available
            if let Some(logs) = response.value.logs {
                println!("\nSimulation logs (first 10):");
                for log in logs.iter().take(10) {
                    println!("  {}", log);
                }
            }
        }
        unit_ok!()
    }

    #[tokio::test]
    async fn generate_solana_validator_cmd() -> AResult<()> {
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
