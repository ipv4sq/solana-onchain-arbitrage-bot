use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use crate::constants::helpers::ToPubkey;

#[derive(Debug)]
pub struct PumpAmmInfo {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub coin_creator_vault_authority: Pubkey,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct PumpPoolData {
    discriminator: [u8; 8],
    _padding1: u8,
    _padding2: [u8; 2],
    _padding3: [u8; 32],
    base_mint: [u8; 32],
    quote_mint: [u8; 32],
    _lp_supply: [u8; 8],
    _padding4: [u8; 24],
    pool_base_token_account: [u8; 32],
    pool_quote_token_account: [u8; 32],
    _padding5: [u8; 8],
    coin_creator: [u8; 32],
}

impl PumpAmmInfo {
    pub fn load_checked(data: &[u8]) -> Result<Self> {
        const EXPECTED_MIN_SIZE: usize = std::mem::size_of::<PumpPoolData>();

        if data.len() < EXPECTED_MIN_SIZE {
            return Err(anyhow::anyhow!(
                "Invalid data length for PumpAmmInfo. Expected at least {} bytes, got {}",
                EXPECTED_MIN_SIZE,
                data.len()
            ));
        }

        // Use bytemuck to safely cast the byte slice to our struct
        let pool_data = bytemuck::try_from_bytes::<PumpPoolData>(&data[..EXPECTED_MIN_SIZE])
            .map_err(|e| anyhow::anyhow!("Failed to parse PumpPoolData: {:?}", e))?;

        let base_mint = Pubkey::new_from_array(pool_data.base_mint);
        let quote_mint = Pubkey::new_from_array(pool_data.quote_mint);
        let pool_base_token_account = Pubkey::new_from_array(pool_data.pool_base_token_account);
        let pool_quote_token_account = Pubkey::new_from_array(pool_data.pool_quote_token_account);

        // Extract coin creator and derive vault authority
        let coin_creator = if pool_data.coin_creator == [0u8; 32] {
            Pubkey::default()
        } else {
            Pubkey::new_from_array(pool_data.coin_creator)
        };

        let pump_program_id = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA".to_pubkey();

        let (coin_creator_vault_authority, _) = Pubkey::find_program_address(
            &[b"creator_vault", coin_creator.as_ref()],
            &pump_program_id,
        );

        Ok(Self {
            base_mint,
            quote_mint,
            pool_base_token_account,
            pool_quote_token_account,
            coin_creator_vault_authority,
        })
    }

    /// Returns (sol_vault, token_vault) based on which position SOL is in
    pub fn get_vaults_for_sol(&self, sol_mint: &Pubkey) -> Option<(Pubkey, Pubkey)> {
        if *sol_mint == self.base_mint {
            Some((self.pool_base_token_account, self.pool_quote_token_account))
        } else if *sol_mint == self.quote_mint {
            Some((self.pool_quote_token_account, self.pool_base_token_account))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::helpers::ToPubkey;
    use crate::test::test_utils::{get_test_rpc_client, pool_addresses};

    #[test]
    fn test_pump_amm_info_parsing_with_rpc() {
        let rpc_client = get_test_rpc_client();

        // Pool address from the JSON data
        let pool_address = pool_addresses::PUMP_TEST_POOL;
        let pool_pubkey = pool_address.to_pubkey();

        // Fetch the actual account data from RPC
        let account = rpc_client
            .get_account(&pool_pubkey)
            .expect("Failed to fetch pool account");

        // Parse using our improved method
        let amm_info =
            PumpAmmInfo::load_checked(&account.data).expect("Failed to parse PumpAmmInfo");

        // Expected values from the JSON data provided
        let expected_base_mint = "34HDZNbUkTyTrgYKy2ox43yp2f8PJ5hoM7xsrfNApump".to_pubkey();
        let expected_quote_mint = "So11111111111111111111111111111111111111112".to_pubkey();
        let expected_pool_base_token = "A3m372MVEeNqyH4PcRBwU4ocoBFEd9vUhFnPL2fgvjeT".to_pubkey();
        let expected_pool_quote_token = "DQf9F5ou9ut1SJ9T4umEGbYzxUrBG4DRPAG9E7Ejg1Mh".to_pubkey();

        // Verify all parsed values match expected
        assert_eq!(amm_info.base_mint, expected_base_mint, "Base mint mismatch");
        assert_eq!(
            amm_info.quote_mint, expected_quote_mint,
            "Quote mint mismatch"
        );
        assert_eq!(
            amm_info.pool_base_token_account, expected_pool_base_token,
            "Pool base token account mismatch"
        );
        assert_eq!(
            amm_info.pool_quote_token_account, expected_pool_quote_token,
            "Pool quote token account mismatch"
        );

        let pump_program_id = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA".to_pubkey();
        let coin_creator = "8bQWTNarKfF7yLa1EMuoNCVZCCgKeB9DonF5u8JciCP4".to_pubkey();
        let (expected_vault_authority, _) = Pubkey::find_program_address(
            &[b"creator_vault", coin_creator.as_ref()],
            &pump_program_id,
        );

        assert_eq!(
            amm_info.coin_creator_vault_authority, expected_vault_authority,
            "Coin creator vault authority mismatch"
        );
    }
}
