use solana_sdk::pubkey::Pubkey;
use crate::constants::addresses::TOKEN_2022_KEY;

pub fn is_spl_token_program(program_id: &Pubkey) -> bool {
    *program_id == spl_token::ID || *program_id == *TOKEN_2022_KEY
}