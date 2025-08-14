use crate::arb::constant::known_pool_program::RAYDIUM_CPMM_PROGRAM;
use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct ExampleAccountData {}

impl PoolAccountDataLoader for ExampleAccountData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        // Raydium CPMM accounts have an 8-byte discriminator at the beginning
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        ExampleAccountData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn get_base_mint(&self) -> Pubkey {
        todo!()
    }

    fn get_quote_mint(&self) -> Pubkey {
        todo!()
    }

    fn get_base_vault(&self) -> Pubkey {
        todo!()
    }

    fn get_quote_vault(&self) -> Pubkey {
        todo!()
    }
}

type ExamplePoolConfig = PoolConfig<ExampleAccountData>;
pub struct ExamplePoolSwapAccounts;
impl PoolConfigInit<ExampleAccountData, ExamplePoolSwapAccounts> for ExamplePoolConfig {
    fn init(pool: &Pubkey, account_data: ExampleAccountData, desired_mint: Pubkey) -> Result<Self> {
        todo!()
    }

    fn build_accounts(&self, payer: &Pubkey, input_mint: &Pubkey, output_mint: &Pubkey) -> Result<ExamplePoolSwapAccounts> {
        todo!()
    }
}

impl ExampleAccountData {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_data() {
        todo!()
    }

    const POOL_ADDRESS: &str = "77Qwh7cW6YqbGSdf8xdao8Sg3QcCyL9g6UmXMxu31VMq";

    const ACCOUNT_DATA_BASE64: &str = "8ZptBBGxbbwALTEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFAAUAAAAAAABAAAAAAAAAGCk3AB5BwAAAQAKAHgAiBNrj51oAAAAAMsQx7q4jQYAAAAAAAAAAAAEkAxJI4JQBwAAAAAAAAAAjUEiAAAAAAAAAAAAAAAAAE1wFQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWLpTSluSJYWp2oCUUkNjDPHqHzIgbewUiIXXvddFbc8Gm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAdpnBXSWDVI3MP0X0BHMpYZ3ebA2CGkz4osfcYUTrRqmzYErpduJrpYGaaVuj6OMDH/5myCcGUm6WEuIkRzFaW4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANpjaB9yhrzIBnGeLCtQogFXJDv8lmixFSC8U4Q+3NsIPAjGaGcsn0zMDQGHMAMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIkGZicKAAAAAAAAAAAAAAAAAAAAAAAAAFA7AQABAAAAAAAAAAAAAACbV2lOqRpchLHE/v8AAAAA2GOVUKucSAcAAAAAAAAAALhHnWgAAAAAAQABAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAOvuthT/5Q0OAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAnc2ViSOR21UNVTn/HwMAAAAAAAAAAAAAAAAAAAAAAACAa7w1LQAAAAAAAAAAAAAAAAAAAAAAAACJBmYnCgAAAAAAAAAAAAAAAAAAAAAAAAADAAAAAAAAAAAAAAAAAAAA2mNoH3KGvMgGcZ4sK1CiAVckO/yWaLEVILxThD7c2wgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    const ACCOUNT_DATA_JSON: &str = r#"
    
{"pool_fees":{"type":{"defined":{"name":"PoolFeesStruct"}},"data":{"base_fee":{"type":{"defined":{"name":"BaseFeeStruct"}},"data":{"cliff_fee_numerator":"20000000","fee_scheduler_mode":0,"padding_0":[0,0,0,0,0],"number_of_period":0,"period_frequency":"0","reduction_factor":"0","padding_1":"0"}},"protocol_fee_percent":{"type":"u8","data":20},"partner_fee_percent":{"type":"u8","data":0},"referral_fee_percent":{"type":"u8","data":20},"padding_0":{"type":{"array":["u8",5]},"data":[0,0,0,0,0]},"dynamic_fee":{"type":{"defined":{"name":"DynamicFeeStruct"}},"data":{"initialized":1,"padding":[0,0,0,0,0,0,0],"max_volatility_accumulator":14460000,"variable_fee_control":1913,"bin_step":1,"filter_period":10,"decay_period":120,"reduction_factor":5000,"last_update_timestamp":"1755156331","bin_step_u128":"1844674407370955","sqrt_price_reference":"527064244463374340","volatility_accumulator":"2245005","volatility_reference":"1405005"}},"padding_1":{"type":{"array":["u64",2]},"data":["0","0"]}}},"token_a_mint":{"type":"pubkey","data":"6yMfYLz3F2n4SzbZFMaB4ra2BhxqjNq7bwG5XNKNATNn"},"token_b_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"token_a_vault":{"type":"pubkey","data":"FhYzTek7L6XbvXm1phUUfqNBAyg9xixZd2Eq3RNRALNq"},"token_b_vault":{"type":"pubkey","data":"EqCpPgDbh5JMek7X4UpNPX7qK6HampyVJFyUm4wDg7UZ"},"whitelisted_vault":{"type":"pubkey","data":"11111111111111111111111111111111"},"partner":{"type":"pubkey","data":"FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM"},"liquidity":{"type":"u128","data":"64691962362050821924071850510396"},"_padding":{"type":"u128","data":"0"},"protocol_a_fee":{"type":"u64","data":"0"},"protocol_b_fee":{"type":"u64","data":"43610670729"},"partner_a_fee":{"type":"u64","data":"0"},"partner_b_fee":{"type":"u64","data":"0"},"sqrt_min_price":{"type":"u128","data":"4295048016"},"sqrt_max_price":{"type":"u128","data":"79226673521066979257578248091"},"sqrt_price":{"type":"u128","data":"524841616193971160"},"activation_point":{"type":"u64","data":"1755137976"},"activation_type":{"type":"u8","data":1},"pool_status":{"type":"u8","data":0},"token_a_flag":{"type":"u8","data":1},"token_b_flag":{"type":"u8","data":0},"collect_fee_mode":{"type":"u8","data":1},"pool_type":{"type":"u8","data":0},"_padding_0":{"type":{"array":["u8",2]},"data":[0,0]},"fee_a_per_liquidity":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"fee_b_per_liquidity":{"type":{"array":["u8",32]},"data":[235,238,182,20,255,229,13,14,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"permanent_lock_liquidity":{"type":"u128","data":"63382289836820512178602402762141"},"metrics":{"type":{"defined":{"name":"PoolMetrics"}},"data":{"total_lp_a_fee":{"type":"u128","data":"0"},"total_lp_b_fee":{"type":"u128","data":"194175069056"},"total_protocol_a_fee":{"type":"u64","data":"0"},"total_protocol_b_fee":{"type":"u64","data":"43610670729"},"total_partner_a_fee":{"type":"u64","data":"0"},"total_partner_b_fee":{"type":"u64","data":"0"},"total_position":{"type":"u64","data":"3"},"padding":{"type":"u64","data":"0"}}},"creator":{"type":"pubkey","data":"FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM"},"_padding_1":{"type":{"array":["u64",6]},"data":["0","0","0","0","0","0"]},"reward_infos":{"type":{"array":[{"defined":{"name":"RewardInfo"}},2]},"data":[{"initialized":0,"reward_token_flag":0,"_padding_0":[0,0,0,0,0,0],"_padding_1":[0,0,0,0,0,0,0,0],"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","reward_per_token_stored":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"},{"initialized":0,"reward_token_flag":0,"_padding_0":[0,0,0,0,0,0],"_padding_1":[0,0,0,0,0,0,0,0],"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","reward_per_token_stored":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"}]}}

    "#;
}
