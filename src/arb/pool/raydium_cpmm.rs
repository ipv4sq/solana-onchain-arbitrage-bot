use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use solana_program::pubkey::Pubkey;

pub struct RaydiumCpmmAccountData {}

impl PoolAccountDataLoader for RaydiumCpmmAccountData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        todo!()
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

type RaydiumCpmmPoolConfig = PoolConfig<RaydiumCpmmAccountData>;
impl PoolConfigInit<RaydiumCpmmAccountData> for RaydiumCpmmPoolConfig {
    fn init(
        pool: &Pubkey,
        account_data: RaydiumCpmmAccountData,
        desired_mint: Pubkey,
    ) -> anyhow::Result<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_data() {
        todo!()
    }


    const POOL_ADDRESS: &str = "8ztFxjFPfVUtEf4SLSapcFj8GW2dxyUA9no2bLPq7H7V";
    const ACCOUNT_DATA_BASE64: &str = "IQsxYrVlsQ0QJx4AWAKIEyBOAAAwVwUAtar//0tVAAD0AQAAAAAAAKlQAACJAgAAxgAAAAAAAACthZ1oAAAAAAAAAAAAAAAA/RQAA8gAAAAUAAAAECcAAMDwQqqsn4I3RswQ4QTTY1WRzy16NHGubwcnwI/bJ02FBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAFLIKeIqVbCJLfRzAXj0i57dYiuNT0BDGidCwPV101ZK/JBTOrKFnQ0+pYZB/CAnCaFTYy4e2m7WYU+0HudVPia3HxfclcAAADNXYQ0OAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2890xm4z7dNMN2joFKm10GFDBVccWYrFno7Rmd3nVw0AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2O7//wEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABULzqvBPVQwYswClugFtsyU938tEfqaHFwk5hbiY2f6AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACZ6kZcnXYJnUKGAndLzPGvshsD4aJ6oRLteCWio9S9RQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
    const ACCOUNT_DATA_JSON: &str = r#"
    {"parameters":{"type":{"defined":{"name":"StaticParameters"}},"data":{"base_factor":{"type":"u16","data":"10000"},"filter_period":{"type":"u16","data":"30"},"decay_period":{"type":"u16","data":"600"},"reduction_factor":{"type":"u16","data":"5000"},"variable_fee_control":{"type":"u32","data":"20000"},"max_volatility_accumulator":{"type":"u32","data":"350000"},"min_bin_id":{"type":"i32","data":"-21835"},"max_bin_id":{"type":"i32","data":"21835"},"protocol_share":{"type":"u16","data":"500"},"base_fee_power_factor":{"type":"u8","data":0},"_padding":{"type":{"array":["u8",5]},"data":[0,0,0,0,0]}}},"v_parameters":{"type":{"defined":{"name":"VariableParameters"}},"data":{"volatility_accumulator":{"type":"u32","data":"20649"},"volatility_reference":{"type":"u32","data":"649"},"index_reference":{"type":"i32","data":"198"},"_padding":{"type":{"array":["u8",4]},"data":[0,0,0,0]},"last_update_timestamp":{"type":"i64","data":"1755153837"},"_padding_1":{"type":{"array":["u8",8]},"data":[0,0,0,0,0,0,0,0]}}},"bump_seed":{"type":{"array":["u8",1]},"data":[253]},"bin_step_seed":{"type":{"array":["u8",2]},"data":[20,0]},"pair_type":{"type":"u8","data":3},"active_id":{"type":"i32","data":"200"},"bin_step":{"type":"u16","data":"20"},"status":{"type":"u8","data":0},"require_base_factor_seed":{"type":"u8","data":0},"base_factor_seed":{"type":{"array":["u8",2]},"data":[16,39]},"activation_type":{"type":"u8","data":0},"creator_pool_on_off_control":{"type":"u8","data":0},"token_x_mint":{"type":"pubkey","data":"Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk"},"token_y_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"reserve_x":{"type":"pubkey","data":"64GTWbkiCgZt62EMccjFHRoT1MQAQviDioa63NCj37w8"},"reserve_y":{"type":"pubkey","data":"HJfR4mh9Yctrrh8pQQsrGsNdqV7KfpaaXGSdxGTwoeBK"},"protocol_fee":{"type":{"defined":{"name":"ProtocolFee"}},"data":{"amount_x":{"type":"u64","data":"375581015260"},"amount_y":{"type":"u64","data":"241399258573"}}},"_padding_1":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"reward_infos":{"type":{"array":[{"defined":{"name":"RewardInfo"}},2]},"data":[{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"},{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"}]},"oracle":{"type":"pubkey","data":"Fo3m9HQx8Rv4EMzmKWxe5yjCZMNcB5W5sKNv4pDzRFqe"},"bin_array_bitmap":{"type":{"array":["u64",16]},"data":["0","0","0","0","0","0","0","18441915018640359424","511","0","0","0","0","0","0","0"]},"last_updated_at":{"type":"i64","data":"0"},"_padding_2":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"pre_activation_swap_address":{"type":"pubkey","data":"11111111111111111111111111111111"},"base_key":{"type":"pubkey","data":"2RA1EnEVxWP8TQZhFt2nXuVcrQetFQUgYyGsUBTWUNpR"},"activation_point":{"type":"u64","data":"0"},"pre_activation_duration":{"type":"u64","data":"0"},"_padding_3":{"type":{"array":["u8",8]},"data":[0,0,0,0,0,0,0,0]},"_padding_4":{"type":"u64","data":"0"},"creator":{"type":"pubkey","data":"BMpa9wWzZepEgp7qxps9G72AnAwfFEQCWxboaNhop1BA"},"token_mint_x_program_flag":{"type":"u8","data":0},"token_mint_y_program_flag":{"type":"u8","data":0},"_reserved":{"type":{"array":["u8",22]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}}
    "#;
}
