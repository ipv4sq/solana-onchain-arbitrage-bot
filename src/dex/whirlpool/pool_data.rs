use crate::dex::interface::PoolDataLoader;
use crate::lined_err;
use crate::util::alias::AResult;
use crate::util::serde_helpers;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Default, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority: Pubkey,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub emissions_per_second_x64: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub growth_global_x64: u128,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct WhirlpoolPoolData {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub fee_tier_index_seed: [u8; 2],
    pub fee_rate: u16,
    pub protocol_fee_rate: u16,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub liquidity: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,
    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub fee_growth_global_a: u128,
    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub fee_growth_global_b: u128,
    pub reward_last_updated_timestamp: u64,
    pub reward_infos: [WhirlpoolRewardInfo; 3],
}

impl PoolDataLoader for WhirlpoolPoolData {
    fn load_data(data: &[u8]) -> AResult<Self> {
        if data.len() < 8 {
            return Err(lined_err!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        let mut data_slice = &data[8..];

        // Use Borsh deserialize which doesn't require all bytes to be consumed
        <WhirlpoolPoolData as BorshDeserialize>::deserialize(&mut data_slice)
            .map_err(|e| lined_err!("Failed to parse account data: {}", e))
    }

    fn base_mint(&self) -> Pubkey {
        self.token_mint_a
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_mint_b
    }

    fn base_vault(&self) -> Pubkey {
        self.token_vault_a
    }

    fn quote_vault(&self) -> Pubkey {
        self.token_vault_b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde_json::Value;
    use solana_sdk::pubkey;

    const POOL: Pubkey = pubkey!("HyA4ct7i4XvZsVrLyb5VJhcTP1EZVDZoF9fFGym16zcj");

    #[tokio::test]
    async fn test_load() {
        let base64_data: &str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf8EAAQEkAEUBQzQhBjTGAAAAAAAAAAAAABZRcwACQaJFAAAAAAAAAAA4Dr//ylJ/5zJAgAA3Wn2FQMAAAAMRfffjZ5ylWKEkz9tmLdXAy6D34RgT7XhF//2HVsS+arsxH6LolpQyA7ZB2HrKJh35uJAUSLFiOx5OciOAW4je0S/FbWsvtQBAAAAAAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hQLp+tKau8fev2FzHuaF6UbkPxSqFcxgDXBXbbUFyiJS7qtCY6ifVAQAAAAAAAAAA0FzFaAAAAAAMANCv64YU2n8Zq6AtQPGMaSWF9lAg387T1eX5qcDE4bCb4EusmZI8S8jOoPk6gpuvm/s4CbScVhpu5GcYqjU7vR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAADWQ2S9J4CwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let expected_json: &str = r#"
        {"whirlpoolsConfig":{"type":"publicKey","data":"2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ"},"whirlpoolBump":{"type":{"array":["u8",1]},"data":[255]},"tickSpacing":{"type":"u16","data":"4"},"feeTierIndexSeed":{"type":{"array":["u8",2]},"data":[4,4]},"feeRate":{"type":"u16","data":"400"},"protocolFeeRate":{"type":"u16","data":"1300"},"liquidity":{"type":"u128","data":"27294928523276"},"sqrtPrice":{"type":"u128","data":"1479720588305778009"},"tickCurrentIndex":{"type":"i32","data":"-50464"},"protocolFeeOwedA":{"type":"u64","data":"3064945658153"},"protocolFeeOwedB":{"type":"u64","data":"13253372381"},"tokenMintA":{"type":"publicKey","data":"pumpCmXqMfrsAkQ5r49WcJnRayYRqmXz6ae8H7H9Dfn"},"tokenVaultA":{"type":"publicKey","data":"CWDi1WBqLTVTkGQNQBxTdLTHnfHwzH88x5ADjbUmKN6W"},"feeGrowthGlobalA":{"type":"u128","data":"33776624149079213179"},"tokenMintB":{"type":"publicKey","data":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"},"tokenVaultB":{"type":"publicKey","data":"5Mg2j1oUCfAWk2YcVirxyUA3js5zgHSyNFpsFACwBrxj"},"feeGrowthGlobalB":{"type":"u128","data":"132055652616940219"},"rewardLastUpdatedTimestamp":{"type":"u64","data":"1757764816"},"rewardInfos":{"type":{"array":[{"defined":"WhirlpoolRewardInfo"},3]},"data":[{"mint":"orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE","vault":"CtQcXLjm1sUhWMtRAxCC9nTR5cU4D7FBKjmtAT2B7dEA","authority":"DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW","emissionsPerSecondX64":"0","growthGlobalX64":"3229069344138253"},{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","authority":"DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW","emissionsPerSecondX64":"0","growthGlobalX64":"0"},{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","authority":"DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW","emissionsPerSecondX64":"0","growthGlobalX64":"0"}]}}
        "#;

        // Decode base64 to bytes
        let pool_data_bytes = STANDARD
            .decode(base64_data)
            .expect("Failed to decode base64");

        // Load pool data using the PoolDataLoader trait
        let pool_data =
            WhirlpoolPoolData::load_data(&pool_data_bytes).expect("Failed to parse pool data");

        // Parse the JSON to verify against
        let json: Value = serde_json::from_str(expected_json).expect("Failed to parse JSON");

        // Test whirlpools config
        assert_eq!(
            pool_data.whirlpools_config.to_string(),
            json["whirlpoolsConfig"]["data"].as_str().unwrap(),
            "Whirlpools config mismatch"
        );

        // Test bump
        assert_eq!(
            pool_data.whirlpool_bump[0],
            json["whirlpoolBump"]["data"][0].as_u64().unwrap() as u8,
            "Whirlpool bump mismatch"
        );

        // Test tick spacing and fee tier
        assert_eq!(
            pool_data.tick_spacing,
            json["tickSpacing"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            "Tick spacing mismatch"
        );
        assert_eq!(
            pool_data.fee_tier_index_seed[0],
            json["feeTierIndexSeed"]["data"][0].as_u64().unwrap() as u8,
            "Fee tier index seed[0] mismatch"
        );
        assert_eq!(
            pool_data.fee_tier_index_seed[1],
            json["feeTierIndexSeed"]["data"][1].as_u64().unwrap() as u8,
            "Fee tier index seed[1] mismatch"
        );

        // Test fees
        assert_eq!(
            pool_data.fee_rate,
            json["feeRate"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            "Fee rate mismatch"
        );
        assert_eq!(
            pool_data.protocol_fee_rate,
            json["protocolFeeRate"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            "Protocol fee rate mismatch"
        );

        // Test liquidity and price
        assert_eq!(
            pool_data.liquidity.to_string(),
            json["liquidity"]["data"].as_str().unwrap(),
            "Liquidity mismatch"
        );
        assert_eq!(
            pool_data.sqrt_price.to_string(),
            json["sqrtPrice"]["data"].as_str().unwrap(),
            "Sqrt price mismatch"
        );
        assert_eq!(
            pool_data.tick_current_index,
            json["tickCurrentIndex"]["data"]
                .as_str()
                .unwrap()
                .parse::<i32>()
                .unwrap(),
            "Tick current index mismatch"
        );

        // Test protocol fees
        assert_eq!(
            pool_data.protocol_fee_owed_a,
            json["protocolFeeOwedA"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap(),
            "Protocol fee owed A mismatch"
        );
        assert_eq!(
            pool_data.protocol_fee_owed_b,
            json["protocolFeeOwedB"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap(),
            "Protocol fee owed B mismatch"
        );

        // Test token mints
        assert_eq!(
            pool_data.token_mint_a.to_string(),
            json["tokenMintA"]["data"].as_str().unwrap(),
            "Token mint A mismatch"
        );
        assert_eq!(
            pool_data.token_mint_b.to_string(),
            json["tokenMintB"]["data"].as_str().unwrap(),
            "Token mint B mismatch"
        );

        // Test token vaults
        assert_eq!(
            pool_data.token_vault_a.to_string(),
            json["tokenVaultA"]["data"].as_str().unwrap(),
            "Token vault A mismatch"
        );
        assert_eq!(
            pool_data.token_vault_b.to_string(),
            json["tokenVaultB"]["data"].as_str().unwrap(),
            "Token vault B mismatch"
        );

        // Test fee growth
        assert_eq!(
            pool_data.fee_growth_global_a.to_string(),
            json["feeGrowthGlobalA"]["data"].as_str().unwrap(),
            "Fee growth global A mismatch"
        );
        assert_eq!(
            pool_data.fee_growth_global_b.to_string(),
            json["feeGrowthGlobalB"]["data"].as_str().unwrap(),
            "Fee growth global B mismatch"
        );

        // Test reward last updated timestamp
        assert_eq!(
            pool_data.reward_last_updated_timestamp,
            json["rewardLastUpdatedTimestamp"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap(),
            "Reward last updated timestamp mismatch"
        );

        // Test reward infos
        let reward_infos = &json["rewardInfos"]["data"];
        for i in 0..3 {
            let reward = &pool_data.reward_infos[i];
            let json_reward = &reward_infos[i];

            assert_eq!(
                reward.mint.to_string(),
                json_reward["mint"].as_str().unwrap(),
                "Reward {} mint mismatch",
                i
            );
            assert_eq!(
                reward.vault.to_string(),
                json_reward["vault"].as_str().unwrap(),
                "Reward {} vault mismatch",
                i
            );
            assert_eq!(
                reward.authority.to_string(),
                json_reward["authority"].as_str().unwrap(),
                "Reward {} authority mismatch",
                i
            );
            assert_eq!(
                reward.emissions_per_second_x64.to_string(),
                json_reward["emissionsPerSecondX64"].as_str().unwrap(),
                "Reward {} emissions per second mismatch",
                i
            );
            assert_eq!(
                reward.growth_global_x64.to_string(),
                json_reward["growthGlobalX64"].as_str().unwrap(),
                "Reward {} growth global mismatch",
                i
            );
        }

        // Test trait methods work correctly
        assert_eq!(pool_data.base_mint(), pool_data.token_mint_a);
        assert_eq!(pool_data.quote_mint(), pool_data.token_mint_b);
        assert_eq!(pool_data.base_vault(), pool_data.token_vault_a);
        assert_eq!(pool_data.quote_vault(), pool_data.token_vault_b);

        println!("✓ All fields match the JSON data!");
        println!("✓ Pool: {}", POOL);
        println!("✓ token_mint_a: {}", pool_data.token_mint_a);
        println!("✓ token_mint_b: {}", pool_data.token_mint_b);
        println!("✓ liquidity: {}", pool_data.liquidity);
        println!("✓ sqrt_price: {}", pool_data.sqrt_price);
        println!("✓ tick_current_index: {}", pool_data.tick_current_index);
    }
}
