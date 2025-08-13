use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct MeteoraDAmmPool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
    pub token_x_token_vault: Pubkey,
    pub token_sol_token_vault: Pubkey,
    pub token_x_lp_mint: Pubkey,
    pub token_sol_lp_mint: Pubkey,
    pub token_x_pool_lp: Pubkey,
    pub token_sol_pool_lp: Pubkey,
    pub admin_token_fee_x: Pubkey,
    pub admin_token_fee_sol: Pubkey,
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

#[derive(Debug, Clone)]
pub struct MeteoraDAmmV2Pool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

#[derive(Debug, Clone)]
pub struct MeteoraDlmmPool {
    pub pair: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub oracle: Pubkey,
    pub bin_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}