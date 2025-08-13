use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct PumpPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub fee_token_wallet: Pubkey,
    pub coin_creator_vault_ata: Pubkey,
    pub coin_creator_vault_authority: Pubkey,
    pub token_mint: Pubkey,
    /// Strange here, 这个地方, 在pool里, base是土狗, quote是sol, 但是在这里base是sol, 可能是想要
    /// 按照sol来套利
    pub base_mint: Pubkey,
}