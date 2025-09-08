use crate::dex::interface::PoolDataLoader;
use crate::util::alias::AResult;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct RaydiumCpmmPoolData {
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
    pub creator_fee_on: u8,
    pub enable_creator_fee: bool,
    pub padding1: [u8; 6],
    pub creator_fees_token_0: u64,
    pub creator_fees_token_1: u64,
    pub padding: [u64; 28],
}

impl PoolDataLoader for RaydiumCpmmPoolData {
    fn load_data(data: &[u8]) -> AResult<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        RaydiumCpmmPoolData::try_from_slice(&data[8..])
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

#[cfg(test)]
mod tests {
    use crate::dex::interface::PoolDataLoader;
    use crate::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
    use crate::util::traits::pubkey::ToPubkey;
    use base64::engine::general_purpose::STANDARD;

    #[tokio::test]
    async fn test_pool_data() {
        let pool = "BtGUffMEnxrzdjyC3kKAHjGMpG1UdZiVWXZUaSpUv13C".to_pubkey();
        let pool_data_json = r#"{
  "amm_config": {
    "type": "pubkey",
    "data": "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2"
  },
  "pool_creator": {
    "type": "pubkey",
    "data": "36DWP52MVRDooYNrcRVDyoCh2R1fPXCYqKJQYg9pFQoE"
  },
  "token_0_vault": {
    "type": "pubkey",
    "data": "SvbJANoKJmz6RqEJBj5gjPrfurkKzhfGXUvaEams48y"
  },
  "token_1_vault": {
    "type": "pubkey",
    "data": "FgPdQQ37kZVDqsfgPSLzC851mx9BMP6HRYe2ia4HDNLe"
  },
  "lp_mint": {
    "type": "pubkey",
    "data": "32G1zhdfadicaoi8Fpw1a7niNqjHsPC718FXZ3Qg5Df3"
  },
  "token_0_mint": {
    "type": "pubkey",
    "data": "So11111111111111111111111111111111111111112"
  },
  "token_1_mint": {
    "type": "pubkey",
    "data": "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk"
  },
  "token_0_program": {
    "type": "pubkey",
    "data": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  },
  "token_1_program": {
    "type": "pubkey",
    "data": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  },
  "observation_key": {
    "type": "pubkey",
    "data": "369Hj5tT85pGAwsUESErUeZybnk5cTBoJ1tysDbs4eM9"
  },
  "auth_bump": {
    "type": "u8",
    "data": 253
  },
  "status": {
    "type": "u8",
    "data": 0
  },
  "lp_mint_decimals": {
    "type": "u8",
    "data": 9
  },
  "mint_0_decimals": {
    "type": "u8",
    "data": 9
  },
  "mint_1_decimals": {
    "type": "u8",
    "data": 6
  },
  "lp_supply": {
    "type": "u64",
    "data": "3836966203034"
  },
  "protocol_fees_token_0": {
    "type": "u64",
    "data": "49023014"
  },
  "protocol_fees_token_1": {
    "type": "u64",
    "data": "3414305342"
  },
  "fund_fees_token_0": {
    "type": "u64",
    "data": "4037703"
  },
  "fund_fees_token_1": {
    "type": "u64",
    "data": "231412299"
  },
  "open_time": {
    "type": "u64",
    "data": "1756490919"
  },
  "recent_epoch": {
    "type": "u64",
    "data": "841"
  },
  "creator_fee_on": {
    "type": "u8",
    "data": 1
  },
  "enable_creator_fee": {
    "type": "bool",
    "data": true
  },
  "padding1": {
    "type": {
      "array": [
        "u8",
        6
      ]
    },
    "data": [
      0,
      0,
      0,
      0,
      0,
      0
    ]
  },
  "creator_fees_token_0": {
    "type": "u64",
    "data": "59466681730"
  },
  "creator_fees_token_1": {
    "type": "u64",
    "data": "0"
  },
  "padding": {
    "type": {
      "array": [
        "u64",
        28
      ]
    },
    "data": [
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0",
      "0"
    ]
  }
}
        "#;
        let data_base64 = "9+3j9dfD3kazIT+6i/nIf6keR4GWKMOD4AvqfpjHoD4DuhBpz8P28x8Na0rc7C1Zm7LyJl4ShisCRi0+5a8Nk4OaqdH0m7wVBqQduY0Igai9wR2ia3vG3PgpwHivUyv22iEjyfHy/CraGtqK7nhH5z24aSL2iI6mFwcGnviAEXfRG0m3q1L/5x4J70KbYQNB95Fy7IrHByfvAX2wY0tJu+3I7ItHuCA+BpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAExB+pJ3KsjugmehWQPi7KSS7v4rbB+qi46TyOqDmQA9Qbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpBt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkfCKZLj4dOgvwYS5t4w4t0gbKegIyNEYUvo/IxnasrnP0ACQkGmu4BXX0DAAAmCOwCAAAAAD4qgssAAAAAR5w9AAAAAABLEssNAAAAAKfssWgAAAAASQMAAAAAAAABAQAAAAAAAIKNfdgNAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
        let pool_data_bytes = STANDARD
            .decode(data_base64)
            .expect("Failed to decode base64");

        // Decode base64 to check for discriminator
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(data_base64)
            .unwrap();

        println!("Decoded data length: {} bytes", decoded.len());

        let loaded = RaydiumCpmmPoolData::load_data(&decoded).unwrap();

        println!("Loaded amm_config: {:?}", loaded.amm_config);
        println!(
            "Expected amm_config: {:?}",
            "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2".to_pubkey()
        );

        assert_eq!(
            loaded.amm_config,
            "D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2".to_pubkey()
        );
        assert_eq!(
            loaded.pool_creator,
            "36DWP52MVRDooYNrcRVDyoCh2R1fPXCYqKJQYg9pFQoE".to_pubkey()
        );
        assert_eq!(
            loaded.token_0_vault,
            "SvbJANoKJmz6RqEJBj5gjPrfurkKzhfGXUvaEams48y".to_pubkey()
        );
        assert_eq!(
            loaded.token_1_vault,
            "FgPdQQ37kZVDqsfgPSLzC851mx9BMP6HRYe2ia4HDNLe".to_pubkey()
        );
        assert_eq!(
            loaded.lp_mint,
            "32G1zhdfadicaoi8Fpw1a7niNqjHsPC718FXZ3Qg5Df3".to_pubkey()
        );
        assert_eq!(
            loaded.token_0_mint,
            "So11111111111111111111111111111111111111112".to_pubkey()
        );
        assert_eq!(
            loaded.token_1_mint,
            "4JPyh4ATbE8hfcH7LqhxF3YThsECZm6htmLvMUyrbonk".to_pubkey()
        );
        assert_eq!(
            loaded.token_0_program,
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_pubkey()
        );
        assert_eq!(
            loaded.token_1_program,
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_pubkey()
        );
        assert_eq!(
            loaded.observation_key,
            "369Hj5tT85pGAwsUESErUeZybnk5cTBoJ1tysDbs4eM9".to_pubkey()
        );
        assert_eq!(loaded.auth_bump, 253);
        assert_eq!(loaded.status, 0);
        assert_eq!(loaded.lp_mint_decimals, 9);
        assert_eq!(loaded.mint_0_decimals, 9);
        assert_eq!(loaded.mint_1_decimals, 6);
        assert_eq!(loaded.lp_supply, 3836966203034);
        assert_eq!(loaded.protocol_fees_token_0, 49023014);
        assert_eq!(loaded.protocol_fees_token_1, 3414305342);
        assert_eq!(loaded.fund_fees_token_0, 4037703);
        assert_eq!(loaded.fund_fees_token_1, 231412299);
        assert_eq!(loaded.open_time, 1756490919);
        assert_eq!(loaded.recent_epoch, 841);
        assert_eq!(loaded.creator_fee_on, 1);
        assert_eq!(loaded.enable_creator_fee, true);
        assert_eq!(loaded.padding1, [0, 0, 0, 0, 0, 0]);
        assert_eq!(loaded.creator_fees_token_0, 59466681730);
        assert_eq!(loaded.creator_fees_token_1, 0);
    }
}
