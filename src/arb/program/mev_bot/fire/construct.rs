use crate::arb::convention::chain::util::alt::fetch_address_lookup_tables;
use crate::arb::convention::chain::util::simulation::SimulationResult;
use crate::arb::global::constant::mev_bot::MevBot;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::state::rpc::{rpc_client, simulate_tx_with_retry};
use crate::arb::convention::pool::interface::{InputAccountUtil, PoolDataLoader};
use crate::arb::convention::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::convention::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::convention::pool::util::{ata, ata_sol_token};
use crate::constants::addresses::SPL_TOKEN_KEY;
use crate::constants::helpers::{ToAccountMeta, ToPubkey};
use crate::util::random_select;
use anyhow::{anyhow, Result};
use solana_program::address_lookup_table::AddressLookupTableAccount;
use solana_program::hash::Hash;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::message::v0::Message;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::VersionedTransaction;
use tracing::info;
use crate::arb::global::constant::token_program::TokenProgram;

const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 500_000;
const DEFAULT_UNIT_PRICE: u64 = 500_000;

pub async fn build_and_send(
    wallet: &Keypair,
    minor_mint: &Pubkey,
    compute_unit_limit: u32,
    unit_price: u64,
    pools: &[AnyPoolConfig],
    minimum_profit: u64,
) -> Result<SimulationResult> {
    let blockhash = rpc_client().get_latest_blockhash().await?;

    let alt_keys = vec![
        // this seems to be legit
        "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
        "7Y77q5Ym5VNsAjY1amGfYGjXUSLjFcgmF6WxeeemiR8T".to_pubkey(),
        "EyFCXwfjTjYAZz7pz1fwiQfRq8YPUKotSNyCeihHMWgZ".to_pubkey(),
        // "q52amtQzHcXs2PA3c4Xqv1LRRZCbFMzd4CGHu1tHdp1".to_pubkey(),
    ];

    let alts = fetch_address_lookup_tables(&alt_keys).await?;

    let tx = build_tx(
        wallet,
        minor_mint,
        compute_unit_limit,
        unit_price,
        pools,
        blockhash,
        &alts,
        minimum_profit,
    )
    .await?;

    // let signature = send_tx_with_retry(&tx, 3).await?;
    // println!("Transaction sent: {}", signature);

    Ok(simulate_tx_with_retry(&tx, 3).await?)
}

pub async fn build_tx(
    wallet: &Keypair,
    minor_mint: &Pubkey,
    compute_unit_limit: u32,
    unit_price: u64,
    pools: &[AnyPoolConfig],
    blockhash: Hash,
    alts: &[AddressLookupTableAccount],
    minimum_profit: u64,
) -> Result<VersionedTransaction> {
    let (mut instructions, limit) = gas_instructions(compute_unit_limit, unit_price);
    let x = pools.first().unwrap();
    let swap_ix = create_invoke_mev_instruction(
        &wallet.pubkey(),
        minor_mint,
        compute_unit_limit,
        pools,
        minimum_profit,
    )?;
    instructions.push(swap_ix);

    let message = Message::try_compile(&wallet.pubkey(), &instructions, alts, blockhash)?;
    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[wallet],
    )?;
    Ok(tx)
}

pub fn create_invoke_mev_instruction(
    signer: &Pubkey,
    minor_mint: &Pubkey,
    compute_unit_limit: u32,
    pools: &[AnyPoolConfig],
    minimum_profit: u64,
) -> Result<Instruction> {
    let use_flashloan = true;
    let fee_account = fee_collector(use_flashloan);
    let mut accounts = vec![
        signer.to_signer(),
        Mints::WSOL.to_readonly(),
        fee_account.to_writable(),
        ata_sol_token(&signer, &Mints::WSOL).to_writable(),
        TokenProgram::SPL_TOKEN.to_program(),
        system_program::ID.to_readonly(),
        spl_associated_token_account::ID.to_readonly(),
    ];

    if use_flashloan {
        accounts.extend([
            MevBot::FLASHLOAN_ACCOUNT.to_readonly(),
            derive_vault_token_account(
                &MevBot::EMV_BOT_PROGRAM,
                &Mints::WSOL, // default to wsol mint base for flashloan
            )
            .0
            .to_writable(),
        ]);
    }
    accounts.extend([
        minor_mint.to_readonly(),
        //TODO  below both will be updated to the real token program
        TokenProgram::SPL_TOKEN.to_program(),
        ata(signer, minor_mint, &TokenProgram::SPL_TOKEN.to_pubkey()).to_writable(),
    ]);

    // let the_other_mint_account = ata(&signer(), )
    for pool in pools {
        match pool {
            AnyPoolConfig::MeteoraDlmm(c) => {
                let built = MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
                    signer, &c.pool, &c.data,
                )?;
                accounts.extend(vec![
                    built.program,
                    c.data.desired_mint()?.to_readonly(),
                    built.event_authority,
                    built.lb_pair,
                    built.reverse_x,
                    built.reverse_y,
                    built.oracle,
                ]);
                accounts.extend(built.bin_arrays);
            }
            AnyPoolConfig::MeteoraDammV2(c) => {
                let built = MeteoraDammV2InputAccount::build_accounts_no_matter_direction_size(
                    signer, &c.pool, &c.data,
                )?;
                accounts.extend(vec![
                    built.meteora_program,
                    c.data.desired_mint()?.to_readonly(),
                    built.event_authority,
                    built.pool_authority,
                    c.pool.to_writable(),
                    built.token_a_vault,
                    built.token_b_vault,
                ]);
            }
            AnyPoolConfig::Unsupported => return Err(anyhow!("Unsupported pool type")),
        };
    }

    // Create instruction data
    let mut data = vec![28u8];

    // When true, the bot will not fail the transaction even when it can't find a profitable arbitrage. It will just do nothing and succeed.
    let no_failure_mode = false;

    data.extend_from_slice(&minimum_profit.to_le_bytes());
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    data.extend_from_slice(if no_failure_mode { &[1] } else { &[0] });
    data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    data.extend_from_slice(if use_flashloan { &[1] } else { &[0] });

    info!("printing our all the accounts");
    accounts.iter().for_each(|account| {
        println!(
            "account: {}, signer: {}, writable: {}",
            account.pubkey, account.is_signer, account.is_writable
        )
    });
    info!("finished printing our all the accounts");

    Ok(Instruction {
        program_id: MevBot::EMV_BOT_PROGRAM,
        accounts,
        data,
    })
}

/// I am not sure whether this would work
pub fn derive_vault_token_account(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault_token_account", mint.as_ref()], program_id)
}

fn fee_collector(use_flashloan: bool) -> Pubkey {
    if use_flashloan {
        MevBot::FLASHLOAN_FEE_ACCOUNT
    } else {
        let fee_accounts = [
            MevBot::NON_FLASHLOAN_ACCOUNT_1,
            MevBot::NON_FLASHLOAN_ACCOUNT_2,
            MevBot::NON_FLASHLOAN_ACCOUNT_3,
        ];
        *random_select(&fee_accounts).expect("fee_accounts should not be empty")
    }
}

fn gas_instructions(compute_limit: u32, unit_price: u64) -> (Vec<Instruction>, u32) {
    let seed = rand::random::<u32>() % 1000;
    let compute_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(compute_limit + seed);
    // 1 lamport = 1_000_000
    let unit_price_ix = ComputeBudgetInstruction::set_compute_unit_price(unit_price);

    (vec![compute_limit_ix, unit_price_ix], compute_limit + seed)
}
