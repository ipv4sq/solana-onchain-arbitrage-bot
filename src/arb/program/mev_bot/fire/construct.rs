use crate::arb::chain::util::alt::fetch_address_lookup_tables;
use crate::arb::constant::mev_bot::{SmbFeeCollector, EMV_BOT_PROGRAM_ID, FLASHLOAN_ACCOUNT_ID};
use crate::arb::constant::mint::{Mints, WSOL_KEY};
use crate::arb::global::rpc::{rpc_client, simulate_tx_with_retry};
use crate::arb::pool::interface::InputAccountUtil;
use crate::arb::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
use crate::arb::pool::register::AnyPoolConfig;
use crate::arb::pool::util::ata_sol_token;
use crate::constants::addresses::TokenProgram;
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

const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 500_000;
const DEFAULT_UNIT_PRICE: u64 = 500_000;

pub async fn build_and_send(
    wallet: &Keypair,
    compute_unit_limit: u32,
    unit_price: u64,
    pools: Vec<AnyPoolConfig>,
    minimum_profit: u64,
) -> Result<()> {
    let rpc = rpc_client();
    let blockhash = rpc.get_latest_blockhash().await?;

    let alt_keys = vec![
        "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
        "q52amtQzHcXs2PA3c4Xqv1LRRZCbFMzd4CGHu1tHdp1".to_pubkey(),
    ];

    let alts = fetch_address_lookup_tables(&rpc, &alt_keys).await?;

    let tx = build_tx(
        wallet,
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

    simulate_tx_with_retry(&tx, 3).await?;

    Ok(())
}

pub async fn build_tx(
    wallet: &Keypair,
    compute_unit_limit: u32,
    unit_price: u64,
    pools: Vec<AnyPoolConfig>,
    blockhash: Hash,
    alts: &[AddressLookupTableAccount],
    minimum_profit: u64,
) -> Result<VersionedTransaction> {
    let (mut instructions, limit) = gas_instructions(compute_unit_limit, unit_price);
    let swap_ix = create_invoke_mev_instruction(wallet, compute_unit_limit, pools, minimum_profit);
    instructions.push(swap_ix?);

    let message = Message::try_compile(&wallet.pubkey(), &instructions, alts, blockhash)?;
    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[wallet],
    )?;
    Ok(tx)
}

fn create_invoke_mev_instruction(
    wallet: &Keypair,
    compute_unit_limit: u32,
    pools: Vec<AnyPoolConfig>,
    minimum_profit: u64,
) -> Result<Instruction> {
    let use_flashloan = true;
    let fee_account = fee_collector(use_flashloan);
    let mut accounts = vec![
        wallet.pubkey().to_signer(),
        Mints::WSOL.to_readonly(),
        fee_account.to_writable(),
        ata_sol_token(&wallet.pubkey(), &WSOL_KEY).to_writable(),
        TokenProgram::SPL_TOKEN.to_readonly(),
        system_program::ID.to_readonly(),
        spl_associated_token_account::ID.to_writable(),
    ];

    if use_flashloan {
        accounts.extend([
            FLASHLOAN_ACCOUNT_ID.to_readonly(),
            derive_vault_token_account(
                &EMV_BOT_PROGRAM_ID.to_pubkey(),
                &Mints::WSOL.to_pubkey(), // default to wsol mint base for flashloan
            )
            .0
            .to_writable(),
        ]);
    }
    // let the_other_mint_account = ata(&wallet.pubkey(), )
    for pool in pools {
        let pool_specific_accounts: Vec<AccountMeta> = match pool {
            AnyPoolConfig::MeteoraDlmm(c) => {
                MeteoraDlmmInputAccounts::build_accounts_no_matter_direction_size(
                    &wallet.pubkey(),
                    &c.pool,
                    &c.data,
                )?
                .to_list_cloned()
            }
            AnyPoolConfig::MeteoraDammV2(c) => {
                MeteoraDammV2InputAccount::build_accounts_no_matter_direction_size(
                    &wallet.pubkey(),
                    &c.pool,
                    &c.data,
                )?
                .to_list_cloned()
            }
            AnyPoolConfig::Unsupported => return Err(anyhow!("Unsupported pool type")),
        };
        accounts.extend(pool_specific_accounts);
    }

    // Create instruction data
    let mut data = vec![28u8];

    // When true, the bot will not fail the transaction even when it can't find a profitable arbitrage. It will just do nothing and succeed.
    let no_failure_mode = true;

    data.extend_from_slice(&minimum_profit.to_le_bytes());
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    data.extend_from_slice(if no_failure_mode { &[1] } else { &[0] });
    data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    data.extend_from_slice(if use_flashloan { &[1] } else { &[0] });

    Ok(Instruction {
        program_id: EMV_BOT_PROGRAM_ID.to_pubkey(),
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
        SmbFeeCollector::FLASHLOAN_FEE_ID.to_pubkey()
    } else {
        let fee_accounts = [
            SmbFeeCollector::NON_FLASHLOAN_FEE_ID_1.to_pubkey(),
            SmbFeeCollector::NON_FLASHLOAN_FEE_ID_2.to_pubkey(),
            SmbFeeCollector::NON_FLASHLOAN_FEE_ID_3.to_pubkey(),
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

