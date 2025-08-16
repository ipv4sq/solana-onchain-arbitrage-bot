use solana_sdk::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
    pub instruction_index: usize,
}

#[derive(Debug, Clone)]
pub struct InnerInstructions {
    pub parent_index: u8,
    pub instructions: Vec<Instruction>,
}