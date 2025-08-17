use crate::arb::chain::instruction::Instruction;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;



impl Instruction {
    fn to_sol_token_transfer_checked(&self, pool_includes_sub: Vec<Pubkey>) -> bool {
        if self.accounts != 4 {
            return false;
        }

        todo!()
    }
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

pub fn create_account_meta(ix: &Instruction, index: usize) -> anyhow::Result<AccountMeta> {
    ix.accounts
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing account at index {}", index))
}
