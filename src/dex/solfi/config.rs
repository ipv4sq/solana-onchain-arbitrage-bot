use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct SolfiPool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}