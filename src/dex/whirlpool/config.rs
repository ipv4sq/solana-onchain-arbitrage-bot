use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct WhirlpoolPool {
    pub pool: Pubkey,
    pub oracle: Pubkey,
    pub x_vault: Pubkey,
    pub y_vault: Pubkey,
    pub tick_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}