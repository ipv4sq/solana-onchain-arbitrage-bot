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
    use solana_program::pubkey;
    use solana_sdk::pubkey::Pubkey;
    use solana_transaction_status::UiTransactionEncoding;
    use std::hash::Hash;

    static POOL: Pubkey = pubkey!("5rCf1DM8LjKTw4YqhnoLcngyZYeNnQqztScTogYHAS6");

    #[tokio::test]
    async fn verify_meteora_dlmm() {
        must_init_db().await;
        // _set_test_client();
    }

    #[tokio::test]
    async fn simulate() -> AResult<()> {
        _set_test_client();
        let config = MeteoraDlmmConfig::from_address(&POOL).await?;
        let payer = "BMnT51N4iSNhWU5PyFFgWwFvN1jgaiiDr9ZHgnkm3iLJ".to_pubkey();
        let accounts = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
            &payer,
            &POOL,
            &config.pool_data,
        )
        .await?
        .to_list_cloned();
        let tx = build_tx(payer, accounts).await?;
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
                        addresses: vec![payer.to_string()],
                    }),
                    min_context_slot: None,
                    inner_instructions: true,
                },
            )
            .await?;
        println!("Transaction Hash: {:?}", response.value.units_consumed);
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
