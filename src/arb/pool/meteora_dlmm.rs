use crate::arb::constant::known_pool_program::{KnownPoolPrograms, METEORA_DLMM_PROGRAM};
use crate::arb::pool::interface::{
    PoolAccountDataLoader, PoolConfig, PoolConfigInit, SwapAccountsToList,
};
use crate::constants::addresses::{TokenProgram, SPL_TOKEN_KEY};
use crate::constants::helpers::{ToAccountMeta, ToPubkey};
use crate::dex::meteora::constants::METEORA_DLMM_PROGRAM_ID;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use itertools::concat;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

const DLMM_EVENT_AUTHORITY: &str = "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6";

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct MeteoraDlmmAccountData {
    pub parameters: StaticParameters,
    pub v_parameters: VariableParameters,
    pub bump_seed: [u8; 1],
    pub bin_step_seed: [u8; 2],
    pub pair_type: u8,
    pub active_id: i32,
    pub bin_step: u16,
    pub status: u8,
    pub require_base_factor_seed: u8,
    pub base_factor_seed: [u8; 2],
    pub activation_type: u8,
    pub creator_pool_on_off_control: u8,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub protocol_fee: ProtocolFee,
    pub _padding_1: [u8; 32],
    pub reward_infos: [RewardInfo; 2],
    pub oracle: Pubkey,
    pub bin_array_bitmap: [u64; 16],
    pub last_updated_at: i64,
    pub _padding_2: [u8; 32],
    pub pre_activation_swap_address: Pubkey,
    pub base_key: Pubkey,
    pub activation_point: u64,
    pub pre_activation_duration: u64,
    pub _padding_3: [u8; 8],
    pub _padding_4: u64,
    pub creator: Pubkey,
    pub token_mint_x_program_flag: u8,
    pub token_mint_y_program_flag: u8,
    pub _reserved: [u8; 22],
}

impl PoolAccountDataLoader for MeteoraDlmmAccountData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        MeteoraDlmmAccountData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn get_base_mint(&self) -> Pubkey {
        self.token_x_mint
    }

    fn get_quote_mint(&self) -> Pubkey {
        self.token_y_mint
    }

    fn get_base_vault(&self) -> Pubkey {
        self.reserve_x
    }

    fn get_quote_vault(&self) -> Pubkey {
        self.reserve_y
    }
}

type MeteoraDlmmPoolConfig = PoolConfig<MeteoraDlmmAccountData>;

#[derive(Debug, Clone, PartialEq)]
pub struct MeteoraDlmmSwapAccounts {
    lb_pair: AccountMeta,
    bin_array_bitmap_extension: AccountMeta,
    reverse_x: AccountMeta,
    reverse_y: AccountMeta,
    user_token_in: AccountMeta,
    user_token_out: AccountMeta,
    token_x_mint: AccountMeta,
    token_y_mint: AccountMeta,
    oracle: AccountMeta,
    host_fee_in: AccountMeta,
    user: AccountMeta,
    token_x_program: AccountMeta,
    token_y_program: AccountMeta,
    event_authority: AccountMeta,
    program: AccountMeta,
    bin_arrays: Vec<AccountMeta>,
}

impl SwapAccountsToList for MeteoraDlmmSwapAccounts {
    fn to_list(&self) -> Vec<&AccountMeta> {
        concat(vec![
            vec![
                &self.lb_pair,
                &self.bin_array_bitmap_extension,
                &self.reverse_x,
                &self.reverse_y,
                &self.user_token_in,
                &self.user_token_out,
                &self.token_x_mint,
                &self.token_y_mint,
                &self.oracle,
                &self.host_fee_in,
                &self.user,
                &self.token_x_program,
                &self.token_y_program,
                &self.event_authority,
                &self.program,
            ],
            self.bin_arrays.iter().collect(),
        ])
    }
}

impl PoolConfigInit<MeteoraDlmmAccountData, MeteoraDlmmSwapAccounts> for MeteoraDlmmPoolConfig {
    fn init(
        pool: &Pubkey,
        account_data: MeteoraDlmmAccountData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(MeteoraDlmmPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
            // readonly_accounts: vec![
            //     //
            //     *METEORA_DLMM_PROGRAM,
            // ],
            // partial_writeable_accounts: concat(vec![
            //     vec![
            //         //
            //         *pool,
            //         account_data.reserve_x,
            //         account_data.reserve_y,
            //     ],
            //     account_data.get_bin_arrays(pool),
            // ]),
        })
    }

    fn build_accounts(
        &self,
        payer: &Pubkey,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<MeteoraDlmmSwapAccounts> {
        Ok(MeteoraDlmmSwapAccounts {
            lb_pair: self.pool.to_writable(),
            bin_array_bitmap_extension: METEORA_DLMM_PROGRAM.to_program(),
            reverse_x: self.data.reserve_x.to_writable(),
            reverse_y: self.data.reserve_y.to_writable(),
            user_token_in: Self::ata(payer, input_mint, &*SPL_TOKEN_KEY).to_writable(),
            user_token_out: Self::ata(payer, output_mint, &*SPL_TOKEN_KEY).to_writable(),
            token_x_mint: input_mint.to_readonly(),
            token_y_mint: output_mint.to_readonly(),
            oracle: self.data.oracle.to_writable(),
            host_fee_in: METEORA_DLMM_PROGRAM.to_program(),
            user: payer.to_signer(),
            token_x_program: SPL_TOKEN_KEY.to_program(),
            token_y_program: SPL_TOKEN_KEY.to_program(),
            event_authority: DLMM_EVENT_AUTHORITY.to_readonly(),
            program: METEORA_DLMM_PROGRAM.to_program(),
            bin_arrays: self
                .data
                .get_bin_arrays(&self.pool)
                .iter()
                .map(|a| a.to_writable())
                .collect(),
        })
    }
}

impl MeteoraDlmmAccountData {
    fn get_bin_arrays(&self, pool: &Pubkey) -> Vec<Pubkey> {
        let mut bin_arrays = Vec::new();

        // Get bin arrays around the active bin
        // Meteora DLMM typically needs 3 bin arrays: previous, current, next
        let active_bin = self.active_id;
        let bin_step = self.bin_step as i32;

        // Bin array indices are calculated based on the active bin ID
        // Each bin array covers a range of bins
        const BINS_PER_ARRAY: i32 = 70; // Standard for Meteora DLMM

        let current_array_index = active_bin / BINS_PER_ARRAY;

        // Add previous, current, and next bin arrays
        for offset in -1..=1 {
            let array_index = current_array_index + offset;
            bin_arrays.push(Self::get_bin_array_pda(pool, array_index));
        }

        bin_arrays
    }

    fn get_bin_array_pda(pool: &Pubkey, bin_array_index: i32) -> Pubkey {
        let index_bytes = bin_array_index.to_le_bytes();
        Pubkey::find_program_address(
            &[b"bin_array", pool.as_ref(), &index_bytes],
            &*METEORA_DLMM_PROGRAM,
        )
        .0
    }
}

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct StaticParameters {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub protocol_share: u16,
    pub base_fee_power_factor: u8,
    pub _padding: [u8; 5],
}

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct VariableParameters {
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub index_reference: i32,
    pub _padding: [u8; 4],
    pub last_update_timestamp: i64,
    pub _padding_1: [u8; 8],
}

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct ProtocolFee {
    pub amount_x: u64,
    pub amount_y: u64,
}

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct RewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub reward_duration: u64,
    pub reward_duration_end: u64,
    pub reward_rate: u128,
    pub last_update_time: u64,
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arb::constant::mint::WSOL_KEY;
    use crate::constants::helpers::ToPubkey;
    use base64::engine::general_purpose;
    use base64::Engine;
    // tx: https://solscan.io/tx/2qVruJuf1dUTnUfG3ePnp4cRSg4WGK3P1AVUaB7MQdEJ7UMnzVdWL2677BNuPJJmowmvmfirEC9XvQ4uPZpcaTxw

    fn load_data() -> Result<MeteoraDlmmAccountData> {
        let data = general_purpose::STANDARD.decode(ACCOUNT_DATA_BASE64)?;
        let account =
            MeteoraDlmmAccountData::load_data(&data).expect("Failed to parse account data");
        return Ok(account);
    }
    #[test]
    fn test_swap_accounts() {
        let payer = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa".to_pubkey();
        let expected = MeteoraDlmmSwapAccounts {
            lb_pair: "8ztFxjFPfVUtEf4SLSapcFj8GW2dxyUA9no2bLPq7H7V".to_writable(),
            bin_array_bitmap_extension: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_program(),
            reverse_x: "64GTWbkiCgZt62EMccjFHRoT1MQAQviDioa63NCj37w8".to_writable(),
            reverse_y: "HJfR4mh9Yctrrh8pQQsrGsNdqV7KfpaaXGSdxGTwoeBK".to_writable(),
            user_token_in: "4m7mnuw9HhbQzK87HNA2NvkinG84M75YZEjbMW8UFaMs".to_writable(),
            user_token_out: "CTyFguG69kwYrzk24P3UuBvY1rR5atu9kf2S6XEwAU8X".to_writable(),
            token_x_mint: "Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk".to_readonly(),
            token_y_mint: "So11111111111111111111111111111111111111112".to_readonly(),
            oracle: "Fo3m9HQx8Rv4EMzmKWxe5yjCZMNcB5W5sKNv4pDzRFqe".to_writable(),
            host_fee_in: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_program(),
            user: "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa".to_signer(),
            token_x_program: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_program(),
            token_y_program: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_program(),
            event_authority: "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6".to_readonly(),
            program: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_program(),
            bin_arrays: vec![
                "9caL9WS3Y1RZ7L3wwXp4qa8hapTicbDY5GJJ3pteP7oX".to_writable(),
                "MrNAjbZvwT2awQDobynRrmkJStE5ejprQ7QmFXLvycq".to_writable(),
                "5Dj2QB9BtRtWV6skbCy6eadj23h6o46CVHpLbjsCJCEB".to_writable(),
                "69EaDEqwjBKKRFKrtRxb7okPDu5EP5nFhbuqrBtekwDg".to_writable(),
                "433yNSNcf1Gx9p8mWATybS81wQtjBfxmrnHpxNUzcMvU".to_writable(),
            ],
        };
        let config =
            MeteoraDlmmPoolConfig::init(&POOL_ADDRESS.to_pubkey(), load_data().unwrap(), *WSOL_KEY)
                .unwrap();

        let result = config.build_accounts(
            &payer,
            &"Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk".to_pubkey(),
            &*WSOL_KEY,
        ).unwrap();
        assert_eq!(result, expected)
    }

    #[test]
    fn test_parse_meteora_dlmm_account_data() {
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::Value;

        let data = general_purpose::STANDARD
            .decode(ACCOUNT_DATA_BASE64)
            .unwrap();
        let account =
            MeteoraDlmmAccountData::load_data(&data).expect("Failed to parse account data");

        let json: Value = serde_json::from_str(ACCOUNT_DATA_JSON).expect("Failed to parse JSON");

        // Verify parameters
        assert_eq!(
            account.parameters.base_factor,
            json["parameters"]["data"]["base_factor"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );
        assert_eq!(
            account.parameters.filter_period,
            json["parameters"]["data"]["filter_period"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );
        assert_eq!(
            account.parameters.protocol_share,
            json["parameters"]["data"]["protocol_share"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );

        // Verify active bin and bin step
        assert_eq!(
            account.active_id,
            json["active_id"]["data"]
                .as_str()
                .unwrap()
                .parse::<i32>()
                .unwrap()
        );
        assert_eq!(
            account.bin_step,
            json["bin_step"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );

        // Verify token mints
        assert_eq!(
            account.token_x_mint.to_string(),
            json["token_x_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.token_y_mint.to_string(),
            json["token_y_mint"]["data"].as_str().unwrap()
        );

        // Verify reserves
        assert_eq!(
            account.reserve_x.to_string(),
            json["reserve_x"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.reserve_y.to_string(),
            json["reserve_y"]["data"].as_str().unwrap()
        );

        // Verify oracle
        assert_eq!(
            account.oracle.to_string(),
            json["oracle"]["data"].as_str().unwrap()
        );
    }

    #[test]
    fn test_bin_arrays() {
        use base64::{engine::general_purpose, Engine as _};

        let pool_pubkey = POOL_ADDRESS.to_pubkey();

        let data = general_purpose::STANDARD
            .decode(ACCOUNT_DATA_BASE64)
            .unwrap();
        let account_data =
            MeteoraDlmmAccountData::load_data(&data).expect("Failed to parse account data");

        // Verify values from JSON
        assert_eq!(account_data.bin_step, 20);
        assert_eq!(account_data.active_id, 200);

        let bin_arrays = account_data.get_bin_arrays(&pool_pubkey);

        // Should have 3 bin arrays (previous, current, next)
        assert_eq!(bin_arrays.len(), 3);

        // Verify the bin array PDAs are being generated
        // Active bin 200 / 70 bins per array = array index 2
        // So we should get arrays for indices 1, 2, 3
        let expected_indices = vec![1, 2, 3];
        for (i, expected_index) in expected_indices.iter().enumerate() {
            let expected_pda =
                MeteoraDlmmAccountData::get_bin_array_pda(&pool_pubkey, *expected_index);
            assert_eq!(bin_arrays[i], expected_pda);
        }
    }

    #[test]
    fn test_get_bin_array_pda() {
        let pool = POOL_ADDRESS.to_pubkey();
        let bin_array_index = 2;

        let pda = MeteoraDlmmAccountData::get_bin_array_pda(&pool, bin_array_index);

        // Verify it's a valid PDA
        let index_bytes = bin_array_index.to_le_bytes();
        let (expected_pda, _bump) = Pubkey::find_program_address(
            &[b"bin_array", pool.as_ref(), &index_bytes],
            &*METEORA_DLMM_PROGRAM,
        );

        assert_eq!(pda, expected_pda);
    }

    const POOL_ADDRESS: &str = "8ztFxjFPfVUtEf4SLSapcFj8GW2dxyUA9no2bLPq7H7V";
    const ACCOUNT_DATA_BASE64: &str = "IQsxYrVlsQ0QJx4AWAKIEyBOAAAwVwUAtar//0tVAAD0AQAAAAAAAKlQAACJAgAAxgAAAAAAAACthZ1oAAAAAAAAAAAAAAAA/RQAA8gAAAAUAAAAECcAAMDwQqqsn4I3RswQ4QTTY1WRzy16NHGubwcnwI/bJ02FBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAFLIKeIqVbCJLfRzAXj0i57dYiuNT0BDGidCwPV101ZK/JBTOrKFnQ0+pYZB/CAnCaFTYy4e2m7WYU+0HudVPia3HxfclcAAADNXYQ0OAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2890xm4z7dNMN2joFKm10GFDBVccWYrFno7Rmd3nVw0AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2O7//wEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABULzqvBPVQwYswClugFtsyU938tEfqaHFwk5hbiY2f6AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACZ6kZcnXYJnUKGAndLzPGvshsD4aJ6oRLteCWio9S9RQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
    const ACCOUNT_DATA_JSON: &str = r#"
    {"parameters":{"type":{"defined":{"name":"StaticParameters"}},"data":{"base_factor":{"type":"u16","data":"10000"},"filter_period":{"type":"u16","data":"30"},"decay_period":{"type":"u16","data":"600"},"reduction_factor":{"type":"u16","data":"5000"},"variable_fee_control":{"type":"u32","data":"20000"},"max_volatility_accumulator":{"type":"u32","data":"350000"},"min_bin_id":{"type":"i32","data":"-21835"},"max_bin_id":{"type":"i32","data":"21835"},"protocol_share":{"type":"u16","data":"500"},"base_fee_power_factor":{"type":"u8","data":0},"_padding":{"type":{"array":["u8",5]},"data":[0,0,0,0,0]}}},"v_parameters":{"type":{"defined":{"name":"VariableParameters"}},"data":{"volatility_accumulator":{"type":"u32","data":"20649"},"volatility_reference":{"type":"u32","data":"649"},"index_reference":{"type":"i32","data":"198"},"_padding":{"type":{"array":["u8",4]},"data":[0,0,0,0]},"last_update_timestamp":{"type":"i64","data":"1755153837"},"_padding_1":{"type":{"array":["u8",8]},"data":[0,0,0,0,0,0,0,0]}}},"bump_seed":{"type":{"array":["u8",1]},"data":[253]},"bin_step_seed":{"type":{"array":["u8",2]},"data":[20,0]},"pair_type":{"type":"u8","data":3},"active_id":{"type":"i32","data":"200"},"bin_step":{"type":"u16","data":"20"},"status":{"type":"u8","data":0},"require_base_factor_seed":{"type":"u8","data":0},"base_factor_seed":{"type":{"array":["u8",2]},"data":[16,39]},"activation_type":{"type":"u8","data":0},"creator_pool_on_off_control":{"type":"u8","data":0},"token_x_mint":{"type":"pubkey","data":"Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk"},"token_y_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"reserve_x":{"type":"pubkey","data":"64GTWbkiCgZt62EMccjFHRoT1MQAQviDioa63NCj37w8"},"reserve_y":{"type":"pubkey","data":"HJfR4mh9Yctrrh8pQQsrGsNdqV7KfpaaXGSdxGTwoeBK"},"protocol_fee":{"type":{"defined":{"name":"ProtocolFee"}},"data":{"amount_x":{"type":"u64","data":"375581015260"},"amount_y":{"type":"u64","data":"241399258573"}}},"_padding_1":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"reward_infos":{"type":{"array":[{"defined":{"name":"RewardInfo"}},2]},"data":[{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"},{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"}]},"oracle":{"type":"pubkey","data":"Fo3m9HQx8Rv4EMzmKWxe5yjCZMNcB5W5sKNv4pDzRFqe"},"bin_array_bitmap":{"type":{"array":["u64",16]},"data":["0","0","0","0","0","0","0","18441915018640359424","511","0","0","0","0","0","0","0"]},"last_updated_at":{"type":"i64","data":"0"},"_padding_2":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"pre_activation_swap_address":{"type":"pubkey","data":"11111111111111111111111111111111"},"base_key":{"type":"pubkey","data":"2RA1EnEVxWP8TQZhFt2nXuVcrQetFQUgYyGsUBTWUNpR"},"activation_point":{"type":"u64","data":"0"},"pre_activation_duration":{"type":"u64","data":"0"},"_padding_3":{"type":{"array":["u8",8]},"data":[0,0,0,0,0,0,0,0]},"_padding_4":{"type":"u64","data":"0"},"creator":{"type":"pubkey","data":"BMpa9wWzZepEgp7qxps9G72AnAwfFEQCWxboaNhop1BA"},"token_mint_x_program_flag":{"type":"u8","data":0},"token_mint_y_program_flag":{"type":"u8","data":0},"_reserved":{"type":{"array":["u8",22]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}}
    "#;
}
