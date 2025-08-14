use crate::arb::constant::known_pool_program::RAYDIUM_CPMM_PROGRAM;
use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use crate::constants::helpers::ToPubkey;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct RaydiumCpmmAccountData {
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

impl PoolAccountDataLoader for RaydiumCpmmAccountData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        // Raydium CPMM accounts have an 8-byte discriminator at the beginning
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        RaydiumCpmmAccountData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn get_base_mint(&self) -> Pubkey {
        self.token_0_mint
    }

    fn get_quote_mint(&self) -> Pubkey {
        self.token_1_mint
    }

    fn get_base_vault(&self) -> Pubkey {
        self.token_0_vault
    }

    fn get_quote_vault(&self) -> Pubkey {
        self.token_1_vault
    }
}

type RaydiumCpmmPoolConfig = PoolConfig<RaydiumCpmmAccountData>;

impl PoolConfigInit<RaydiumCpmmAccountData> for RaydiumCpmmPoolConfig {
    fn init(
        pool: &Pubkey,
        account_data: RaydiumCpmmAccountData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(RaydiumCpmmPoolConfig {
            pool: *pool,
            pool_data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
            readonly_accounts: vec![],
            partial_writeable_accounts: vec![
                *pool,
                RAYDIUM_CPMM_AUTHORITY.to_pubkey(),
                account_data.amm_config,
                account_data.token_0_vault,
                account_data.token_1_vault,
                account_data.observation_key,
            ],
        })
    }
}

const RAYDIUM_CPMM_AUTHORITY: &str = "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::constant::mint::WSOL_KEY;
    use crate::constants::addresses::TokenMint;
    use base64::engine::general_purpose;
    use base64::Engine;
    // tx: https://solscan.io/tx/4mUwr6wFSxmmaThPELhF5WZECS9GLm6DQqBu3fUKQNaMQ8MXUvaykKnmJGfK8MCHMk3xVSTbrMVBnzKrKE3MnRXS

    fn load_data() -> Result<RaydiumCpmmAccountData> {
        let data = general_purpose::STANDARD.decode(ACCOUNT_DATA_BASE64)?;
        let account =
            RaydiumCpmmAccountData::load_data(&data).expect("Failed to parse account data");
        return Ok(account);
    }

    #[test]
    fn test_computed_accounts() {
        let account = load_data().unwrap();
        let expected_writeable = vec![
            // Authority
            RAYDIUM_CPMM_AUTHORITY.to_pubkey(),
            // Amm Config
            "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2".to_pubkey(),
            // Pool itself
            POOL_ADDRESS.to_pubkey(),
            // Input/Output Vault
            "HgNPDD8bpbSrGyHegiCT5xrYxHTfwLfZydwGkjNCJRKA".to_pubkey(),
            "9xsCiNwYQXM3ZeHFSVj9JQdP1vREJREpN23f6wvxA1ty".to_pubkey(),
            // Observation State
            "4UdSz2kMddtX4woMmdgkWg75fdBP8FgYwqfkh4ri7mnD".to_pubkey(),
        ];

        let config =
            RaydiumCpmmPoolConfig::init(&POOL_ADDRESS.to_pubkey(), account, *WSOL_KEY).unwrap();

        crate::test::test_utils::assert_vec_eq_unordered(
            config.partial_writeable_accounts,
            expected_writeable,
        );
    }

    #[test]
    fn test_parse_raydium_cpmm_account_data() {
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::Value;

        let account = load_data().unwrap();

        // Parse the JSON to validate
        let json: Value = serde_json::from_str(ACCOUNT_DATA_JSON).expect("Failed to parse JSON");

        // Verify pubkeys
        assert_eq!(
            account.amm_config.to_string(),
            json["amm_config"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.pool_creator.to_string(),
            json["pool_creator"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_0_vault.to_string(),
            json["token_0_vault"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_1_vault.to_string(),
            json["token_1_vault"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.lp_mint.to_string(),
            json["lp_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_0_mint.to_string(),
            json["token_0_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_1_mint.to_string(),
            json["token_1_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.observation_key.to_string(),
            json["observation_key"]["data"].as_str().unwrap()
        );

        // Verify numeric fields
        assert_eq!(
            account.auth_bump,
            json["auth_bump"]["data"].as_u64().unwrap() as u8
        );
        assert_eq!(
            account.status,
            json["status"]["data"].as_u64().unwrap() as u8
        );
        assert_eq!(
            account.lp_mint_decimals,
            json["lp_mint_decimals"]["data"].as_u64().unwrap() as u8
        );
        assert_eq!(
            account.mint_0_decimals,
            json["mint_0_decimals"]["data"].as_u64().unwrap() as u8
        );
        assert_eq!(
            account.mint_1_decimals,
            json["mint_1_decimals"]["data"].as_u64().unwrap() as u8
        );
        assert_eq!(
            account.lp_supply,
            json["lp_supply"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            account.protocol_fees_token_0,
            json["protocol_fees_token_0"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            account.protocol_fees_token_1,
            json["protocol_fees_token_1"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            account.open_time,
            json["open_time"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            account.recent_epoch,
            json["recent_epoch"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
    }

    static POOL_ADDRESS: &str = "Q2sPHPdUWFMg7M7wwrQKLrn619cAucfRsmhVJffodSp";
    static ACCOUNT_DATA_BASE64: &str = "9+3j9dfD3kazIT+6i/nIf6keR4GWKMOD4AvqfpjHoD4DuhBpz8P28weDqBUmIQ+UoOjoTSCHADs6QmDSnOedrP4hpOp3H6Yo99DR9Nz3HrIvLX+7YqZ2L5QFPcbvqO5j7jgEz1A6FvWFLTQUxi+NcVdpuk2Us1Cb8MiCjK8yI7k/I/2wM8rbvkgG4f/8XSR9OfB6FhamY3bUmPUZIrYgM7RdSto8z863BpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAHA8EKqrJ+CN0bMEOEE02NVkc8tejRxrm8HJ8CP2ydNhQbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpBt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkzpvRZyLWF6nuaHB6SMwzfHdkN/3ZUCsHaqKbnX+PiTv0ACQkGLaITJdADAACoruUcAAAAAGraDhEAAAAAHOJdBwAAAADXe7MEAAAAAFVhH2gAAAAAQQMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
    static ACCOUNT_DATA_JSON: &str = r#"{"amm_config":{"type":"pubkey","data":"D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2"},"pool_creator":{"type":"pubkey","data":"WLHv2UAZm6z4KyaaELi5pjdbJh6RESMva1Rnn8pJVVh"},"token_0_vault":{"type":"pubkey","data":"HgNPDD8bpbSrGyHegiCT5xrYxHTfwLfZydwGkjNCJRKA"},"token_1_vault":{"type":"pubkey","data":"9xsCiNwYQXM3ZeHFSVj9JQdP1vREJREpN23f6wvxA1ty"},"lp_mint":{"type":"pubkey","data":"5rASbyrUYh4eVmZpgN6MxVY2w83dC4PFg9U9WYc9HW7g"},"token_0_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"token_1_mint":{"type":"pubkey","data":"Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk"},"token_0_program":{"type":"pubkey","data":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"},"token_1_program":{"type":"pubkey","data":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"},"observation_key":{"type":"pubkey","data":"4UdSz2kMddtX4woMmdgkWg75fdBP8FgYwqfkh4ri7mnD"},"auth_bump":{"type":"u8","data":253},"status":{"type":"u8","data":0},"lp_mint_decimals":{"type":"u8","data":9},"mint_0_decimals":{"type":"u8","data":9},"mint_1_decimals":{"type":"u8","data":6},"lp_supply":{"type":"u64","data":"4192510124589"},"protocol_fees_token_0":{"type":"u64","data":"484814504"},"protocol_fees_token_1":{"type":"u64","data":"286186090"},"fund_fees_token_0":{"type":"u64","data":"123593244"},"fund_fees_token_1":{"type":"u64","data":"78871511"},"open_time":{"type":"u64","data":"1746886997"},"recent_epoch":{"type":"u64","data":"833"},"padding":{"type":{"array":["u64",31]},"data":["0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0"]}}"#;
}
