use crate::dex::interface::PoolDataLoader;
use crate::util::serde_helpers;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct RewardInfo {
    pub reward_state: u8,
    pub open_time: u64,
    pub end_time: u64,
    pub last_update_time: u64,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub emissions_per_second_x64: u128,
    pub reward_total_emissioned: u64,
    pub reward_claimed: u64,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub authority: Pubkey,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub reward_growth_global_x64: u128,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct RaydiumCammPoolData {
    pub bump: [u8; 1],
    pub amm_config: Pubkey,
    pub owner: Pubkey,
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,
    pub observation_key: Pubkey,
    pub mint_decimals_0: u8,
    pub mint_decimals_1: u8,
    pub tick_spacing: u16,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub liquidity: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub sqrt_price_x64: u128,
    pub tick_current: i32,
    pub padding3: u16,
    pub padding4: u16,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub fee_growth_global_0_x64: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub fee_growth_global_1_x64: u128,
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub swap_in_amount_token_0: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub swap_out_amount_token_1: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub swap_in_amount_token_1: u128,
    #[serde(with = "serde_helpers::u128_as_string")]
    pub swap_out_amount_token_0: u128,
    pub status: u8,
    pub padding: [u8; 7],
    pub reward_infos: [RewardInfo; 3],
    pub tick_array_bitmap: [u64; 16],
    pub total_fees_token_0: u64,
    pub total_fees_claimed_token_0: u64,
    pub total_fees_token_1: u64,
    pub total_fees_claimed_token_1: u64,
    pub fund_fees_token_0: u64,
    pub fund_fees_token_1: u64,
    pub open_time: u64,
    pub recent_epoch: u64,
    pub padding1: [u64; 24],
    pub padding2: [u64; 32],
}

impl PoolDataLoader for RaydiumCammPoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        // Skip the 8-byte discriminator
        let mut data_slice = &data[8..];

        // Use Borsh deserialize which doesn't require all bytes to be consumed
        <RaydiumCammPoolData as BorshDeserialize>::deserialize(&mut data_slice)
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn base_mint(&self) -> Pubkey {
        self.token_mint_0
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_mint_1
    }

    fn base_vault(&self) -> Pubkey {
        self.token_vault_0
    }

    fn quote_vault(&self) -> Pubkey {
        self.token_vault_1
    }
}

#[cfg(test)]
mod test {
    const POOL: &str = "3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv";
    const POOL_DATA_JSON: &str = r#"
    {"bump":{"type":{"array":["u8",1]},"data":[255]},"amm_config":{"type":"pubkey","data":"3h2e43PunVA5K34vwKCLHWhZF4aZpyaC9RmxvshGAQpL"},"owner":{"type":"pubkey","data":"CJKrW95iMGECdjWtdDnWDAx2cBH7pFE9VywnULfwMapf"},"token_mint_0":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"token_mint_1":{"type":"pubkey","data":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"},"token_vault_0":{"type":"pubkey","data":"4ct7br2vTPzfdmY3S5HLtTxcGSBfn6pnw98hsS6v359A"},"token_vault_1":{"type":"pubkey","data":"5it83u57VRrVgc51oNV19TTmAJuffPx5GtGwQr7gQNUo"},"observation_key":{"type":"pubkey","data":"3Y695CuQ8AP4anbwAqiEBeQF9KxqHFr8piEwvw3UePnQ"},"mint_decimals_0":{"type":"u8","data":9},"mint_decimals_1":{"type":"u8","data":6},"tick_spacing":{"type":"u16","data":"1"},"liquidity":{"type":"u128","data":"295329867866867"},"sqrt_price_x64":{"type":"u128","data":"9082733951060270080"},"tick_current":{"type":"i32","data":"-14171"},"padding3":{"type":"u16","data":"0"},"padding4":{"type":"u16","data":"0"},"fee_growth_global_0_x64":{"type":"u128","data":"3559325779701363188"},"fee_growth_global_1_x64":{"type":"u128","data":"626808862733012568"},"protocol_fees_token_0":{"type":"u64","data":"12806897"},"protocol_fees_token_1":{"type":"u64","data":"5455291"},"swap_in_amount_token_0":{"type":"u128","data":"49476625175238063"},"swap_out_amount_token_1":{"type":"u128","data":"8189820303939792"},"swap_in_amount_token_1":{"type":"u128","data":"8230440941799804"},"swap_out_amount_token_0":{"type":"u128","data":"49669505447062833"},"status":{"type":"u8","data":0},"padding":{"type":{"array":["u8",7]},"data":[0,0,0,0,0,0,0]},"reward_infos":{"type":{"array":[{"defined":{"name":"RewardInfo"}},3]},"data":[{"reward_state":2,"open_time":"1756722600","end_time":"1759746600","last_update_time":"1757745628","emissions_per_second_x64":"30500568904943041694000","reward_total_emissioned":"38697441408","reward_claimed":"38144131402","token_mint":"4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R","token_vault":"HsBUudV9Y2Z2dJTieWFgK3zhrpX4ELvnfHcAwSBVqDGX","authority":"NCV2Uo3hfW5LSZXAJe19y6SpC5K98PuQwShCSZgTki3","reward_growth_global_x64":"116300140988083267"},{"reward_state":0,"open_time":"0","end_time":"0","last_update_time":"0","emissions_per_second_x64":"0","reward_total_emissioned":"0","reward_claimed":"0","token_mint":"11111111111111111111111111111111","token_vault":"11111111111111111111111111111111","authority":"CJKrW95iMGECdjWtdDnWDAx2cBH7pFE9VywnULfwMapf","reward_growth_global_x64":"0"},{"reward_state":0,"open_time":"0","end_time":"0","last_update_time":"0","emissions_per_second_x64":"0","reward_total_emissioned":"0","reward_claimed":"0","token_mint":"11111111111111111111111111111111","token_vault":"11111111111111111111111111111111","authority":"CJKrW95iMGECdjWtdDnWDAx2cBH7pFE9VywnULfwMapf","reward_growth_global_x64":"0"}]},"tick_array_bitmap":{"type":{"array":["u64",16]},"data":["13233894920445562880","16138067892758380435","18446744073709551615","18446744073709551615","18446744073709551615","337992908638485503","4516828126648832","1143210130","1073840145","72057594037927936","0","0","0","9223372036854775808","0","0"]},"total_fees_token_0":{"type":"u64","data":"16624154768112"},"total_fees_claimed_token_0":{"type":"u64","data":"16316837962051"},"total_fees_token_1":{"type":"u64","data":"2765436108315"},"total_fees_claimed_token_1":{"type":"u64","data":"2704587280086"},"fund_fees_token_0":{"type":"u64","data":"16530299"},"fund_fees_token_1":{"type":"u64","data":"4757642"},"open_time":{"type":"u64","data":"1723037622"},"recent_epoch":{"type":"u64","data":"848"},"padding1":{"type":{"array":["u64",24]},"data":["0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0"]},"padding2":{"type":{"array":["u64",32]},"data":["0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0"]}}
    "#;
    const POLL_DATA_BASE64: &str = "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";

    #[test]
    pub fn test_load_data() {}
}
