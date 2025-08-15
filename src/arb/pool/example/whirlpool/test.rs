#[cfg(test)]
mod tests {
    use crate::arb::pool::example::whirlpool::data::WhirlpoolPoolData;
    use crate::arb::pool::interface::PoolDataLoader;
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
            WhirlpoolPoolData::load_data(&data).expect("Failed to parse account data");

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
            WhirlpoolPoolData::get_oracle(&POOL_ADDRESS.to_pubkey()),
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
        let account = WhirlpoolPoolData::load_data(&data).expect("Failed to parse account data");

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