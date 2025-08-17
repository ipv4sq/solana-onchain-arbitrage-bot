use crate::arb::chain::data::instruction::Instruction;
use solana_program::instruction::AccountMeta;

impl Instruction {
    
}

pub fn is_program_ix_with_min_accounts<'a>(
    ix: &'a Instruction,
    program_id: &str,
    min_accounts: usize,
) -> Option<&'a Instruction> {
    use crate::constants::helpers::ToPubkey;
    if ix.program_id == program_id.to_pubkey() {
        if ix.accounts.len() >= min_accounts {
            Some(ix)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn create_account_meta(
    ix: &Instruction,
    index: usize,
) -> anyhow::Result<AccountMeta> {
    ix.accounts
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing account at index {}", index))
}