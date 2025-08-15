use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::arb::constant::known_pool_program::PUMP_PROGRAM;
use crate::arb::pool::interface::{PoolDataLoader, PoolConfig};

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct PumpAmmPoolData {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    pub _padding: [u8; 57],
}

impl PoolDataLoader for PumpAmmPoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        PumpAmmPoolData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn get_base_mint(&self) -> Pubkey {
        self.base_mint
    }

    fn get_quote_mint(&self) -> Pubkey {
        self.quote_mint
    }

    fn get_base_vault(&self) -> Pubkey {
        self.pool_base_token_account
    }

    fn get_quote_vault(&self) -> Pubkey {
        self.pool_quote_token_account
    }
}

pub type PumpAmmPoolConfig = PoolConfig<PumpAmmPoolData>;

impl PumpAmmPoolData {
    pub(crate) fn get_creator_vault_authority(coin_creator: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[b"creator_vault", coin_creator.as_ref()],
            &*PUMP_PROGRAM,
        )
        .0
    }

    fn get_creator_vault_ata(vault_authority: &Pubkey, token_mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(vault_authority, token_mint)
    }
}