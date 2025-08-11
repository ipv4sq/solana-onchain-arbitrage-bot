use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::instruction;

pub fn assemble_create_ata_account_ix(
    belong_to: &Pubkey,
    mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    instruction::create_associated_token_account_idempotent(
        belong_to,
        belong_to,
        mint_address,
        token_program_id,
    )
}
