use crate::global::constant::token_program::TokenProgram;
use solana_sdk::pubkey::Pubkey;

pub fn is_spl_token_program(program_id: &Pubkey) -> bool {
    *program_id == spl_token::ID || *program_id == TokenProgram::TOKEN_2022
}
