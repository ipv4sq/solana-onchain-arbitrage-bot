use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::constant::mev_bot::MevBot;
use crate::global::constant::mint::Mints;
use crate::global::constant::token_program::{SystemProgram, TokenProgram};
use crate::util::alias::{MintAddress, TokenProgramAddress};
use crate::util::random::random_choose;
use crate::util::solana::pda::{ata, ata_sol_token};
use crate::util::traits::account_meta::ToAccountMeta;
use anyhow::Result;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;

pub async fn build_mev_ix(
    wallet: &Keypair,
    minor_mint: &Pubkey,
    compute_unit_limit: u32,
    pools: &[AnyPoolConfig],
    minimum_profit: u64,
    never_abort: bool,
    include_create_token_account_ix: bool,
) -> Result<Vec<Instruction>> {
    let (mut instructions, _limit) = compute_limit_ix(compute_unit_limit);

    let wallet_pub = wallet.pubkey();
    let mint_token_program = MintRecordRepository::get_mint_or_err(minor_mint)
        .await?
        .program
        .0;

    if include_create_token_account_ix {
        instructions.push(ensure_token_account_exists(
            &wallet_pub,
            minor_mint,
            &mint_token_program,
        ))
    }

    let swap_ix = create_invoke_mev_instruction(
        &wallet.pubkey(),
        minor_mint,
        &mint_token_program,
        compute_unit_limit,
        pools,
        minimum_profit,
        never_abort,
    )
    .await?;
    instructions.push(swap_ix);

    Ok(instructions)
}

pub async fn create_invoke_mev_instruction(
    signer: &Pubkey,
    minor_mint: &MintAddress,
    token_program: &TokenProgramAddress,
    compute_unit_limit: u32,
    pools: &[AnyPoolConfig],
    minimum_profit: u64,
    never_abort: bool,
) -> Result<Instruction> {
    let use_flashloan = true;
    let fee_account = fee_collector(use_flashloan);
    let mut accounts = vec![
        signer.to_signer(),
        Mints::WSOL.to_readonly(),
        fee_account.to_writable(),
        ata_sol_token(&signer, &Mints::WSOL).to_writable(),
        TokenProgram::SPL_TOKEN.to_program(),
        SystemProgram.to_readonly(),
        spl_associated_token_account::ID.to_readonly(),
    ];

    if use_flashloan {
        accounts.extend([
            MevBot::FLASHLOAN_ACCOUNT.to_readonly(),
            derive_vault_token_account_mev_bot(
                &MevBot::EMV_BOT_PROGRAM,
                &Mints::WSOL, // default to wsol mint base for flashloan
            )
            .0
            .to_writable(),
        ]);
    }

    accounts.extend([
        minor_mint.to_readonly(),
        token_program.to_program(),
        ata(signer, minor_mint, token_program).to_writable(),
    ]);

    for pool in pools {
        let specific_accounts = pool.build_mev_bot_ix_accounts(signer).await?;
        accounts.extend(specific_accounts);
    }

    // Create instruction data
    let mut data = vec![28u8];

    // When true, the bot will not fail the transaction even when it can't find a profitable arbitrage. It will just do nothing and succeed.
    let no_failure_mode = never_abort;

    data.extend_from_slice(&minimum_profit.to_le_bytes());
    data.extend_from_slice(&compute_unit_limit.to_le_bytes());
    data.extend_from_slice(if no_failure_mode { &[1] } else { &[0] });
    data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    data.extend_from_slice(if use_flashloan { &[1] } else { &[0] });

    Ok(Instruction {
        program_id: MevBot::EMV_BOT_PROGRAM,
        accounts,
        data,
    })
}

pub fn derive_vault_token_account_mev_bot(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault_token_account", mint.as_ref()], program_id)
}

fn fee_collector(use_flashloan: bool) -> Pubkey {
    if use_flashloan {
        MevBot::FLASHLOAN_FEE_ACCOUNT
    } else {
        *random_choose(&[
            MevBot::NON_FLASHLOAN_ACCOUNT_1,
            MevBot::NON_FLASHLOAN_ACCOUNT_2,
            MevBot::NON_FLASHLOAN_ACCOUNT_3,
        ])
    }
}

pub fn compute_limit_ix(compute_limit: u32) -> (Vec<Instruction>, u32) {
    let seed = rand::random::<u32>() % 1000;
    let compute_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(compute_limit + seed);
    (vec![compute_limit_ix], compute_limit + seed)
}

fn ensure_token_account_exists(
    belong_to: &Pubkey,
    mint: &Pubkey,
    mint_program: &Pubkey,
) -> Instruction {
    create_associated_token_account_idempotent(belong_to, belong_to, mint, &mint_program)
}
