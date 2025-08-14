use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub emissions_per_second_x64: u128,
    pub growth_global_x64: u128,
}

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
pub struct WhirlpoolAccountData {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub fee_tier_index_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    pub fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_growth_global_b: u128,
    pub reward_last_updated_timestamp: u64,
    pub reward_infos: [WhirlpoolRewardInfo; 3],
}

impl PoolAccountDataLoader for WhirlpoolAccountData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        Ok(WhirlpoolAccountData::try_from_slice(data)?)
    }

    fn get_base_mint(&self) -> Pubkey {
        self.token_mint_a
    }

    fn get_quote_mint(&self) -> Pubkey {
        self.token_mint_b
    }

    fn get_base_vault(&self) -> Pubkey {
        self.token_vault_a
    }

    fn get_quote_vault(&self) -> Pubkey {
        self.token_vault_b
    }
}

type WhirlpoolPoolConfig = PoolConfig<WhirlpoolAccountData>;

impl PoolConfigInit for WhirlpoolPoolConfig {
    fn init<T>(account_data: T, desired_mint: Pubkey) -> anyhow::Result<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_whirlpool_account_data() {
        use base64::{Engine as _, engine::general_purpose};
        
        let base64_data = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf9gAGAAZBkUBd1vfPF2EgAAAAAAAAAAAACMhPiq2k/x3wAAAAAAAAAAjPX//zKFrwAAAAAAldmPAQAAAAAGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAVefegaXBwZMxqwalusK9L5+17+jVTju2F30gp2IvXMa5F+OlBU1zWgAAAAAAAAAAMDwQqqsn4I3RswQ4QTTY1WRzy16NHGubwcnwI/bJ02F2zeUqN1nAxXz4s+kyh7WkgN8R2G4u2rmyBHN2n+YatsWGj41yzbpnQEAAAAAAAAAZGidaAAAAADA8EKqrJ+CN0bMEOEE02NVkc8tejRxrm8HJ8CP2ydNhbmMneuU3O/D1Y36i7YiayN1+Q5zoVMSUb5fcGpxrIKNvR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAAi32hSmVI5C4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        
        let data = general_purpose::STANDARD
            .decode(base64_data)
            .expect("Failed to decode base64");
        
        println!("Data length: {} bytes", data.len());
        println!("Expected struct size: {} bytes", std::mem::size_of::<WhirlpoolAccountData>());
        
        let account = WhirlpoolAccountData::load_data(&data).expect("Failed to parse account data");
        
        // Verify key fields match the JSON
        assert_eq!(account.tick_spacing, 96);
        assert_eq!(account.fee_rate, 6500);
        assert_eq!(account.protocol_fee_rate, 1300);
        assert_eq!(account.tick_current_index, -2676);
        
        // Verify token mints
        assert_eq!(
            account.token_mint_a.to_string(),
            "So11111111111111111111111111111111111112"
        );
        assert_eq!(
            account.token_mint_b.to_string(),
            "Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk"
        );
        
        // Verify vaults
        assert_eq!(
            account.token_vault_a.to_string(),
            "6u3WgGqEEMaMpPWKKvYchsN5VdmNvNgYGHMgnDqV3gcZ"
        );
        assert_eq!(
            account.token_vault_b.to_string(),
            "FkjSckxG7iizjgWJPRWRyiqWYbGxgBWHw9CM7Y5pS5hc"
        );
        
        // Verify reward info
        assert_eq!(
            account.reward_infos[0].mint.to_string(),
            "Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk"
        );
        assert_eq!(
            account.reward_infos[0].vault.to_string(),
            "DVJncL1FBxs1M6zSgBUdcr2U2A1qgACApDpF5UsX3uEU"
        );
    }
}


