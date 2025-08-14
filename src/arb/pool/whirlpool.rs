use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use crate::arb::constant::known_pool_program::WHIRLPOOL_PROGRAM;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use itertools::concat;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
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
        // Whirlpool accounts always have an 8-byte discriminator at the beginning
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        WhirlpoolAccountData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
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

impl PoolConfigInit<WhirlpoolAccountData> for WhirlpoolPoolConfig {
    fn init(
        pool: &Pubkey,
        account_data: WhirlpoolAccountData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(WhirlpoolPoolConfig {
            pool: *pool,
            pool_data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
            readonly_accounts: vec![
                // TODO memo program
                desired_mint,
                *WHIRLPOOL_PROGRAM,
            ],
            writeable_accounts: concat(vec![
                vec![
                    *pool,
                    WhirlpoolAccountData::get_oracle(pool),
                    account_data.token_vault_a,
                    account_data.token_vault_b,
                ],
                account_data.get_tick_arrays(pool),
            ]),
        })
    }
}

impl WhirlpoolAccountData {
    fn get_oracle(pool: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"oracle", pool.as_ref()], &*WHIRLPOOL_PROGRAM).0
    }

    fn get_tick_arrays(&self, pool: &Pubkey) -> Vec<Pubkey> {
        const TICK_ARRAY_SIZE: i32 = 88;

        let tick_spacing = self.tick_spacing as i32;
        let current_tick = self.tick_current_index;
        let num_ticks_in_array = TICK_ARRAY_SIZE * tick_spacing;

        // Calculate start index for current tick array
        let current_start = if current_tick < 0 && current_tick % num_ticks_in_array != 0 {
            current_tick - (current_tick % num_ticks_in_array) - num_ticks_in_array
        } else {
            current_tick - (current_tick % num_ticks_in_array)
        };

        // Get tick arrays for both directions (previous, current, next)
        let prev_start = current_start - num_ticks_in_array;
        let next_start = current_start + num_ticks_in_array;

        vec![
            Self::get_tick_array_pda(pool, prev_start),
            Self::get_tick_array_pda(pool, current_start),
            Self::get_tick_array_pda(pool, next_start),
        ]
    }

    fn get_tick_array_pda(pool: &Pubkey, start_tick_index: i32) -> Pubkey {
        let start_tick_str = start_tick_index.to_string();
        Pubkey::find_program_address(
            &[b"tick_array", pool.as_ref(), start_tick_str.as_bytes()],
            &*WHIRLPOOL_PROGRAM,
        )
        .0
    }
}
#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub emissions_per_second_x64: u128,
    pub growth_global_x64: u128,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::helpers::ToPubkey;

    #[test]
    fn test_tick_arrays() {
        use base64::{engine::general_purpose, Engine as _};

        let expected = vec![
            "GmgiicyLogtstf8u5VPtsQo6ySnQdQFahAVsb6Q1jTT4",
            "CLhHiTrxhiwNF5PXPp5aQrq3GjkUXuReTyNsSaSry9w8",
            "ASdtdhEzLbbkw1zNwPDJnp9swihi8JkJ3BDwDaxnHG8f",
        ];

        // Use the actual test pool from the JSON data
        let pool_pubkey = POOL_ADDRESS.to_pubkey();

        // Parse the actual account data from base64
        let data = general_purpose::STANDARD
            .decode(ACCOUNT_DATA_BASE64)
            .unwrap();
        let account_data =
            WhirlpoolAccountData::load_data(&data).expect("Failed to parse account data");

        // Verify tick values from JSON
        assert_eq!(account_data.tick_spacing, 96);
        assert_eq!(account_data.tick_current_index, -3071);

        let tick_arrays = account_data.get_tick_arrays(&pool_pubkey);

        assert_eq!(tick_arrays.len(), 3);
        assert_eq!(tick_arrays[0].to_string(), expected[0]);
        assert_eq!(tick_arrays[1].to_string(), expected[1]);
        assert_eq!(tick_arrays[2].to_string(), expected[2]);
    }

    #[test]
    fn test_get_oracle() {
        let expected = "6uSvnzqPXLW986gQhVkY2u4xWjx8CEA2B8PAvq4A5N4w";
        assert_eq!(
            WhirlpoolAccountData::get_oracle(&POOL_ADDRESS.to_pubkey()),
            expected.to_pubkey()
        )
    }

    #[test]
    fn test_parse_whirlpool_account_data() {
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::Value;

        let data = general_purpose::STANDARD
            .decode(ACCOUNT_DATA_BASE64)
            .unwrap();
        let account = WhirlpoolAccountData::load_data(&data).expect("Failed to parse account data");

        // Parse the JSON to validate
        let json: Value = serde_json::from_str(ACCOUNT_DATA_JSON).expect("Failed to parse JSON");

        // Verify numeric fields
        assert_eq!(
            account.tick_spacing,
            json["tickSpacing"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );
        assert_eq!(
            account.fee_rate,
            json["feeRate"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );
        assert_eq!(
            account.protocol_fee_rate,
            json["protocolFeeRate"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );
        assert_eq!(
            account.tick_current_index,
            json["tickCurrentIndex"]["data"]
                .as_str()
                .unwrap()
                .parse::<i32>()
                .unwrap()
        );

        // Verify pubkeys
        assert_eq!(
            account.token_mint_a.to_string(),
            json["tokenMintA"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_mint_b.to_string(),
            json["tokenMintB"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_vault_a.to_string(),
            json["tokenVaultA"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_vault_b.to_string(),
            json["tokenVaultB"]["data"].as_str().unwrap()
        );

        // Verify reward info
        let reward_infos = json["rewardInfos"]["data"].as_array().unwrap();
        assert_eq!(
            account.reward_infos[0].mint.to_string(),
            reward_infos[0]["mint"].as_str().unwrap()
        );
        assert_eq!(
            account.reward_infos[0].vault.to_string(),
            reward_infos[0]["vault"].as_str().unwrap()
        );
    }
    static POOL_ADDRESS: &str = "HsQGWEh3ib6w59rBh5n1jXmi8VXFBqKEjxozL6PGfcgb";
    static ACCOUNT_DATA_BASE64: &str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf9gAGAAZBkUBT+JBnSLEgAAAAAAAAAAAABahPv2uI2R2wAAAAAAAAAAAfT//2RyBgAAAAAAAAAAAAAAAAAGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAVefegaXBwZMxqwalusK9L5+17+jVTju2F30gp2IvXMaO953eRHm1mgAAAAAAAAAAMDwQqqsn4I3RswQ4QTTY1WRzy16NHGubwcnwI/bJ02F2zeUqN1nAxXz4s+kyh7WkgN8R2G4u2rmyBHN2n+YatstnJixVyHqnQEAAAAAAAAARm6daAAAAADA8EKqrJ+CN0bMEOEE02NVkc8tejRxrm8HJ8CP2ydNhbmMneuU3O/D1Y36i7YiayN1+Q5zoVMSUb5fcGpxrIKNvR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAAi32hSmVI5C4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    static ACCOUNT_DATA_JSON: &str = r#"{
  "whirlpoolsConfig": {
    "type": "publicKey",
    "data": "2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ"
  },
  "whirlpoolBump": {
    "type": {
      "array": [
        "u8",
        1
      ]
    },
    "data": [
      255
    ]
  },
  "tickSpacing": {
    "type": "u16",
    "data": "96"
  },
  "feeTierIndexSeed": {
    "type": {
      "array": [
        "u8",
        2
      ]
    },
    "data": [
      96,
      0
    ]
  },
  "feeRate": {
    "type": "u16",
    "data": "6500"
  },
  "protocolFeeRate": {
    "type": "u16",
    "data": "1300"
  },
  "liquidity": {
    "type": "u128",
    "data": "20390156339519"
  },
  "sqrtPrice": {
    "type": "u128",
    "data": "15821582791486440538"
  },
  "tickCurrentIndex": {
    "type": "i32",
    "data": "-3071"
  },
  "protocolFeeOwedA": {
    "type": "u64",
    "data": "422500"
  },
  "protocolFeeOwedB": {
    "type": "u64",
    "data": "0"
  },
  "tokenMintA": {
    "type": "publicKey",
    "data": "So11111111111111111111111111111111111111112"
  },
  "tokenVaultA": {
    "type": "publicKey",
    "data": "6u3WgGqEEMaMpPWKKvYchsN5VdmNvNgYGHMgnDqV3gcZ"
  },
  "feeGrowthGlobalA": {
    "type": "u128",
    "data": "7554478387687317051"
  },
  "tokenMintB": {
    "type": "publicKey",
    "data": "Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk"
  },
  "tokenVaultB": {
    "type": "publicKey",
    "data": "FkjSckxG7iizjgWJPRWRyiqWYbGxgBWHw9CM7Y5pS5hc"
  },
  "feeGrowthGlobalB": {
    "type": "u128",
    "data": "29825688142739971117"
  },
  "rewardLastUpdatedTimestamp": {
    "type": "u64",
    "data": "1755147846"
  },
  "rewardInfos": {
    "type": {
      "array": [
        {
          "defined": "WhirlpoolRewardInfo"
        },
        3
      ]
    },
    "data": [
      {
        "mint": "Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk",
        "vault": "DVJncL1FBxs1M6zSgBUdcr2U2A1qgACApDpF5UsX3uEU",
        "authority": "DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW",
        "emissionsPerSecondX64": "0",
        "growthGlobalX64": "3378905220315708811"
      },
      {
        "mint": "11111111111111111111111111111111",
        "vault": "11111111111111111111111111111111",
        "authority": "DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW",
        "emissionsPerSecondX64": "0",
        "growthGlobalX64": "0"
      },
      {
        "mint": "11111111111111111111111111111111",
        "vault": "11111111111111111111111111111111",
        "authority": "DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW",
        "emissionsPerSecondX64": "0",
        "growthGlobalX64": "0"
      }
    ]
  }
}
        "#;
}
