use crate::arb::pool::interface::PoolDataLoader;
use crate::arb::pool::meteora_damm_v2::pool_data_type::{PoolFeesStruct, PoolMetrics, RewardInfo};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct MeteoraDammV2PoolData {
    pub pool_fees: PoolFeesStruct,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub whitelisted_vault: Pubkey,
    pub partner: Pubkey,
    pub liquidity: u128,
    pub _padding: u128,
    pub protocol_a_fee: u64,
    pub protocol_b_fee: u64,
    pub partner_a_fee: u64,
    pub partner_b_fee: u64,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,
    pub activation_type: u8,
    pub pool_status: u8,
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub collect_fee_mode: u8,
    pub pool_type: u8,
    pub _padding_0: [u8; 2],
    pub fee_a_per_liquidity: [u8; 32],
    pub fee_b_per_liquidity: [u8; 32],
    pub permanent_lock_liquidity: u128,
    pub metrics: PoolMetrics,
    pub creator: Pubkey,
    pub _padding_1: [u64; 6],
    pub reward_infos: [RewardInfo; 2],
}

impl PoolDataLoader for MeteoraDammV2PoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        let mut data_slice = &data[8..];

        // Use deserialize which doesn't require all bytes to be consumed
        MeteoraDammV2PoolData::deserialize(&mut data_slice)
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn get_base_mint(&self) -> Pubkey {
        self.token_a_mint
    }

    fn get_quote_mint(&self) -> Pubkey {
        self.token_b_mint
    }

    fn get_base_vault(&self) -> Pubkey {
        self.token_a_vault
    }

    fn get_quote_vault(&self) -> Pubkey {
        self.token_b_vault
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    //pool address: https://solscan.io/account/6CXXieC355gteamwofSzJn8DiyrbKyYyXc3eBKmB81CF#accountData

    const POOL_ADDRESS: &str = "6CXXieC355gteamwofSzJn8DiyrbKyYyXc3eBKmB81CF";
    const POOL_DATA_BASE64:&str = "8ZptBBGxbbwALTEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFAAUAAAAAAABAAAAAAAAAGCk3AB5BwAAAQAKAHgAiBOBw55oAAAAAMsQx7q4jQYAAAAAAAAAAAA63DvuflFcAAAAAAAAAAAA1lUfAAAAAAAAAAAAAAAAAFZ3DwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA3u13fczH3yMaMIsos9faQNXyPusy3spck0u7z7U/NAMGm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAXlv7GbitC70hNydXpTHYu1X6nJVfc9yofMBfYjH3fQIDeCdXF/AikQ7dzlJEG8I/eDUjnzw4ksSm77t5ttrsRUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANpjaB9yhrzIBnGeLCtQogFXJDv8lmixFSC8U4Q+3NsIIMFawaP0pTpKlCR3WXkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMl44MalAAAAAAAAAAAAAAAAAAAAAAAAAFA7AQABAAAAAAAAAAAAAACbV2lOqRpchLHE/v8AAAAAzg9XX2rOXAAAAAAAAAAAACZSnmgAAAAAAQAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEwNJO/+j10GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMrpOqYsstW2/KpsmVG0AAAAAAAAAAAAAAAAAAAAAAAD4w4Jw9QIAAAAAAAAAAAAAAAAAAAAAAADJeODGpQAAAAAAAAAAAAAAAAAAAAAAAAAgAAAAAAAAAAAAAAAAAAAA2mNoH3KGvMgGcZ4sK1CiAVckO/yWaLEVILxThD7c2wgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const POOL_DATA_JSON: &str = r#"
    {"pool_fees":{"type":{"defined":{"name":"PoolFeesStruct"}},"data":{"base_fee":{"type":{"defined":{"name":"BaseFeeStruct"}},"data":{"cliff_fee_numerator":"20000000","fee_scheduler_mode":0,"padding_0":[0,0,0,0,0],"number_of_period":0,"period_frequency":"0","reduction_factor":"0","padding_1":"0"}},"protocol_fee_percent":{"type":"u8","data":20},"partner_fee_percent":{"type":"u8","data":0},"referral_fee_percent":{"type":"u8","data":20},"padding_0":{"type":{"array":["u8",5]},"data":[0,0,0,0,0]},"dynamic_fee":{"type":{"defined":{"name":"DynamicFeeStruct"}},"data":{"initialized":1,"padding":[0,0,0,0,0,0,0],"max_volatility_accumulator":14460000,"variable_fee_control":1913,"bin_step":1,"filter_period":10,"decay_period":120,"reduction_factor":5000,"last_update_timestamp":"1755235201","bin_step_u128":"1844674407370955","sqrt_price_reference":"25985303462009914","volatility_accumulator":"2053590","volatility_reference":"1013590"}},"padding_1":{"type":{"array":["u64",2]},"data":["0","0"]}}},"token_a_mint":{"type":"pubkey","data":"G1DXVVmqJs8Ei79QbK41dpgk2WtXSGqLtx9of7o8BAGS"},"token_b_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"token_a_vault":{"type":"pubkey","data":"9B3KPhHyDhUmNvjY2vk6JYs3vfUgPTzB9u1fWYsfK1s5"},"token_b_vault":{"type":"pubkey","data":"wAx8Her71ffN9hNyh5nj6WR7m56tAGrkajNiEdoGy4G"},"whitelisted_vault":{"type":"pubkey","data":"11111111111111111111111111111111"},"partner":{"type":"pubkey","data":"FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM"},"liquidity":{"type":"u128","data":"2461259741443399418112242871877920"},"_padding":{"type":"u128","data":"0"},"protocol_a_fee":{"type":"u64","data":"0"},"protocol_b_fee":{"type":"u64","data":"712006203593"},"partner_a_fee":{"type":"u64","data":"0"},"partner_b_fee":{"type":"u64","data":"0"},"sqrt_min_price":{"type":"u128","data":"4295048016"},"sqrt_max_price":{"type":"u128","data":"79226673521066979257578248091"},"sqrt_price":{"type":"u128","data":"26122654118776782"},"activation_point":{"type":"u64","data":"1755206182"},"activation_type":{"type":"u8","data":1},"pool_status":{"type":"u8","data":0},"token_a_flag":{"type":"u8","data":0},"token_b_flag":{"type":"u8","data":0},"collect_fee_mode":{"type":"u8","data":1},"pool_type":{"type":"u8","data":0},"_padding_0":{"type":{"array":["u8",2]},"data":[0,0]},"fee_a_per_liquidity":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"fee_b_per_liquidity":{"type":{"array":["u8",32]},"data":[76,13,36,239,254,143,93,6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"permanent_lock_liquidity":{"type":"u128","data":"2217449760464976157620544597834290"},"metrics":{"type":{"defined":{"name":"PoolMetrics"}},"data":{"total_lp_a_fee":{"type":"u128","data":"0"},"total_lp_b_fee":{"type":"u128","data":"3253177861112"},"total_protocol_a_fee":{"type":"u64","data":"0"},"total_protocol_b_fee":{"type":"u64","data":"712006203593"},"total_partner_a_fee":{"type":"u64","data":"0"},"total_partner_b_fee":{"type":"u64","data":"0"},"total_position":{"type":"u64","data":"32"},"padding":{"type":"u64","data":"0"}}},"creator":{"type":"pubkey","data":"FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM"},"_padding_1":{"type":{"array":["u64",6]},"data":["0","0","0","0","0","0"]},"reward_infos":{"type":{"array":[{"defined":{"name":"RewardInfo"}},2]},"data":[{"initialized":0,"reward_token_flag":0,"_padding_0":[0,0,0,0,0,0],"_padding_1":[0,0,0,0,0,0,0,0],"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","reward_per_token_stored":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"},{"initialized":0,"reward_token_flag":0,"_padding_0":[0,0,0,0,0,0],"_padding_1":[0,0,0,0,0,0,0,0],"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","reward_per_token_stored":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"}]}}
    "#;

    pub fn load_pool_data() -> MeteoraDammV2PoolData {
        let pool_data_bytes = STANDARD
            .decode(POOL_DATA_BASE64)
            .expect("Failed to decode base64");
        MeteoraDammV2PoolData::load_data(&pool_data_bytes).expect("Failed to parse pool data")
    }

    #[test]
    pub fn test_meteora_damm_v2_pool_data_parsing() {
        use serde_json::Value;

        let pool_data = load_pool_data();

        // Parse the JSON to verify against
        let json: Value = serde_json::from_str(POOL_DATA_JSON).expect("Failed to parse JSON");

        // Test the critical fields that the PoolAccountDataLoader interface requires
        assert_eq!(
            pool_data.token_a_mint.to_string(),
            json["token_a_mint"]["data"].as_str().unwrap(),
            "Token A mint mismatch"
        );
        assert_eq!(
            pool_data.token_b_mint.to_string(),
            json["token_b_mint"]["data"].as_str().unwrap(),
            "Token B mint mismatch"
        );
        assert_eq!(
            pool_data.token_a_vault.to_string(),
            json["token_a_vault"]["data"].as_str().unwrap(),
            "Token A vault mismatch"
        );
        assert_eq!(
            pool_data.token_b_vault.to_string(),
            json["token_b_vault"]["data"].as_str().unwrap(),
            "Token B vault mismatch"
        );

        // Test trait methods work correctly
        assert_eq!(pool_data.get_base_mint(), pool_data.token_a_mint);
        assert_eq!(pool_data.get_quote_mint(), pool_data.token_b_mint);
        assert_eq!(pool_data.get_base_vault(), pool_data.token_a_vault);
        assert_eq!(pool_data.get_quote_vault(), pool_data.token_b_vault);

        println!("✓ All critical fields match the JSON data!");
        println!("✓ token_a_mint: {}", pool_data.token_a_mint);
        println!("✓ token_b_mint: {}", pool_data.token_b_mint);
        println!("✓ token_a_vault: {}", pool_data.token_a_vault);
        println!("✓ token_b_vault: {}", pool_data.token_b_vault);
    }
}
