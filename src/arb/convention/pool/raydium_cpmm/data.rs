use crate::arb::convention::pool::interface::PoolDataLoader;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct RaydiumCpmmAPoolData {
    pub amm_config: Pubkey,
    pub pool_creator: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub token_0_program: Pubkey,
    pub token_1_program: Pubkey,
    pub observation_key: Pubkey,
    pub auth_bump: u8,
    pub status: u8,
    pub lp_mint_decimals: u8,
    pub mint_0_decimals: u8,
    pub mint_1_decimals: u8,
    pub lp_supply: u64,
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub padding: [u64; 31],
}

impl PoolDataLoader for RaydiumCpmmAPoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        // Raydium CPMM accounts have an 8-byte discriminator at the beginning
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        RaydiumCpmmAPoolData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn base_mint(&self) -> Pubkey {
        self.token_0_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_1_mint
    }

    fn base_vault(&self) -> Pubkey {
        self.token_0_vault
    }

    fn quote_vault(&self) -> Pubkey {
        self.token_1_vault
    }
}
