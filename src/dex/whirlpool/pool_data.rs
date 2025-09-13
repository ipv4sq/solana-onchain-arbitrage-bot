use crate::dex::interface::PoolDataLoader;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct WhirlpoolPoolData {}

impl PoolDataLoader for WhirlpoolPoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        todo!()
    }

    fn base_mint(&self) -> Pubkey {
        todo!()
    }

    fn quote_mint(&self) -> Pubkey {
        todo!()
    }

    fn base_vault(&self) -> Pubkey {
        todo!()
    }

    fn quote_vault(&self) -> Pubkey {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey;
    const POOL: Pubkey = pubkey!("HyA4ct7i4XvZsVrLyb5VJhcTP1EZVDZoF9fFGym16zcj");

    #[tokio::test]
    async fn test_load() {
        let base64_data:&str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf8EAAQEkAEUBQzQhBjTGAAAAAAAAAAAAABZRcwACQaJFAAAAAAAAAAA4Dr//ylJ/5zJAgAA3Wn2FQMAAAAMRfffjZ5ylWKEkz9tmLdXAy6D34RgT7XhF//2HVsS+arsxH6LolpQyA7ZB2HrKJh35uJAUSLFiOx5OciOAW4je0S/FbWsvtQBAAAAAAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hQLp+tKau8fev2FzHuaF6UbkPxSqFcxgDXBXbbUFyiJS7qtCY6ifVAQAAAAAAAAAA0FzFaAAAAAAMANCv64YU2n8Zq6AtQPGMaSWF9lAg387T1eX5qcDE4bCb4EusmZI8S8jOoPk6gpuvm/s4CbScVhpu5GcYqjU7vR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAADWQ2S9J4CwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let expected_json: &str = r#"
        {"whirlpoolsConfig":{"type":"publicKey","data":"2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ"},"whirlpoolBump":{"type":{"array":["u8",1]},"data":[255]},"tickSpacing":{"type":"u16","data":"4"},"feeTierIndexSeed":{"type":{"array":["u8",2]},"data":[4,4]},"feeRate":{"type":"u16","data":"400"},"protocolFeeRate":{"type":"u16","data":"1300"},"liquidity":{"type":"u128","data":"27294928523276"},"sqrtPrice":{"type":"u128","data":"1479720588305778009"},"tickCurrentIndex":{"type":"i32","data":"-50464"},"protocolFeeOwedA":{"type":"u64","data":"3064945658153"},"protocolFeeOwedB":{"type":"u64","data":"13253372381"},"tokenMintA":{"type":"publicKey","data":"pumpCmXqMfrsAkQ5r49WcJnRayYRqmXz6ae8H7H9Dfn"},"tokenVaultA":{"type":"publicKey","data":"CWDi1WBqLTVTkGQNQBxTdLTHnfHwzH88x5ADjbUmKN6W"},"feeGrowthGlobalA":{"type":"u128","data":"33776624149079213179"},"tokenMintB":{"type":"publicKey","data":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"},"tokenVaultB":{"type":"publicKey","data":"5Mg2j1oUCfAWk2YcVirxyUA3js5zgHSyNFpsFACwBrxj"},"feeGrowthGlobalB":{"type":"u128","data":"132055652616940219"},"rewardLastUpdatedTimestamp":{"type":"u64","data":"1757764816"},"rewardInfos":{"type":{"array":[{"defined":"WhirlpoolRewardInfo"},3]},"data":[{"mint":"orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE","vault":"CtQcXLjm1sUhWMtRAxCC9nTR5cU4D7FBKjmtAT2B7dEA","authority":"DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW","emissionsPerSecondX64":"0","growthGlobalX64":"3229069344138253"},{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","authority":"DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW","emissionsPerSecondX64":"0","growthGlobalX64":"0"},{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","authority":"DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW","emissionsPerSecondX64":"0","growthGlobalX64":"0"}]}}
        "#;
    }
}
