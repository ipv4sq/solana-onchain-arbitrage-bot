#[cfg(test)]
mod tests {
    use crate::arb::convention::pool::interface::{PoolConfigInit, PoolDataLoader};
    use crate::arb::convention::pool::meteora_dlmm::bin_array;
    use crate::arb::convention::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;
    use crate::arb::convention::pool::meteora_dlmm::pool_config::*;
    use crate::arb::convention::pool::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
    use crate::arb::global::constant::mint::Mints;
    use crate::arb::util::traits::account_meta::ToAccountMeta;
    use crate::arb::util::traits::pubkey::ToPubkey;
    use anyhow::Result;
    use base64::engine::general_purpose;
    use base64::Engine;
    // tx1: https://solscan.io/tx/2qVruJuf1dUTnUfG3ePnp4cRSg4WGK3P1AVUaB7MQdEJ7UMnzVdWL2677BNuPJJmowmvmfirEC9XvQ4uPZpcaTxw
    // tx2:

    fn load_data() -> Result<MeteoraDlmmPoolData> {
        let data = general_purpose::STANDARD.decode(ACCOUNT_DATA_BASE64)?;
        let account = MeteoraDlmmPoolData::load_data(&data).expect("Failed to parse account data");
        Ok(account)
    }

    #[test]
    fn test_swap_accounts_with_amount() {
        // Test with the actual transaction amount from the swap instruction
        let amount_in = 449_360_555u64;
        let payer = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa".to_pubkey();

        let config = MeteoraDlmmPoolConfig::from_pool_data(
            &POOL_ADDRESS.to_pubkey(),
            load_data().unwrap(),
            Mints::WSOL,
        )
        .unwrap();

        // Build accounts with the specific amount
        let result = config
            .build_accounts_with_amount(
                &payer,
                &"Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk".to_pubkey(),
                &Mints::WSOL,
                amount_in,
            )
            .unwrap();

        // Compare with expected from transaction
        let expected_arrays = vec![
            "9caL9WS3Y1RZ7L3wwXp4qa8hapTicbDY5GJJ3pteP7oX",
            "MrNAjbZvwT2awQDobynRrmkJStE5ejprQ7QmFXLvycq",
            "5Dj2QB9BtRtWV6skbCy6eadj23h6o46CVHpLbjsCJCEB",
            "69EaDEqwjBKKRFKrtRxb7okPDu5EP5nFhbuqrBtekwDg",
            "433yNSNcf1Gx9p8mWATybS81wQtjBfxmrnHpxNUzcMvU",
        ];

        assert_eq!(
            result.bin_arrays.len(),
            5,
            "Large swap should generate 5 bin arrays"
        );

        // Verify the pattern matches the actual transaction order: 2, 1, 0, -1, -2
        let expected_indices = vec![2, 1, 0, -1, -2];

        for (i, expected_idx) in expected_indices.iter().enumerate() {
            let expected_pda =
                bin_array::get_bin_array_pda(&POOL_ADDRESS.to_pubkey(), *expected_idx);
            assert_eq!(
                result.bin_arrays[i].pubkey, expected_pda,
                "Bin array {} should match expected PDA for index {}",
                i, expected_idx
            );
        }
    }

    #[test]
    fn test_swap_accounts() {
        // This test validates the structure of swap accounts
        // Note: bin arrays are dynamic based on the swap size and liquidity distribution
        // The expected values here are from a specific historical transaction
        let payer = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa".to_pubkey();
        let expected = MeteoraDlmmInputAccounts {
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
            event_authority: DLMM_EVENT_AUTHORITY.to_readonly(),
            program: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_program(),
            bin_arrays: vec![
                "9caL9WS3Y1RZ7L3wwXp4qa8hapTicbDY5GJJ3pteP7oX".to_writable(),
                "MrNAjbZvwT2awQDobynRrmkJStE5ejprQ7QmFXLvycq".to_writable(),
                "5Dj2QB9BtRtWV6skbCy6eadj23h6o46CVHpLbjsCJCEB".to_writable(),
                "69EaDEqwjBKKRFKrtRxb7okPDu5EP5nFhbuqrBtekwDg".to_writable(),
                "433yNSNcf1Gx9p8mWATybS81wQtjBfxmrnHpxNUzcMvU".to_writable(),
            ],
        };
        let config = MeteoraDlmmPoolConfig::from_pool_data(
            &POOL_ADDRESS.to_pubkey(),
            load_data().unwrap(),
            Mints::WSOL,
        )
        .unwrap();

        let result = config
            .build_accounts_with_amount(
                &payer,
                &"Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk".to_pubkey(),
                &Mints::WSOL,
                449_360_555u64,
            )
            .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_meteora_dlmm_account_data() {
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::Value;

        let data = general_purpose::STANDARD
            .decode(ACCOUNT_DATA_BASE64)
            .unwrap();
        let account = MeteoraDlmmPoolData::load_data(&data).expect("Failed to parse account data");

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
    fn test_get_bin_array_pda() {
        let pool = POOL_ADDRESS.to_pubkey();

        // Test generating PDAs for different indices
        println!("\n=== Testing Bin Array PDA Generation ===");

        // Generate PDAs for indices -2 to 4 to match expected
        for index in -2..=4 {
            let pda = bin_array::get_bin_array_pda(&pool, index);
            println!("Index {}: {}", index, pda);
        }

        // Check specific expected arrays
        println!("\nChecking expected arrays from transaction:");
        let expected = vec![
            ("9caL9WS3Y1RZ7L3wwXp4qa8hapTicbDY5GJJ3pteP7oX", 2),
            ("MrNAjbZvwT2awQDobynRrmkJStE5ejprQ7QmFXLvycq", 1),
            ("5Dj2QB9BtRtWV6skbCy6eadj23h6o46CVHpLbjsCJCEB", 0),
            ("69EaDEqwjBKKRFKrtRxb7okPDu5EP5nFhbuqrBtekwDg", -1),
            ("433yNSNcf1Gx9p8mWATybS81wQtjBfxmrnHpxNUzcMvU", -2),
        ];

        for (expected_pda, expected_index) in expected {
            let generated_pda = bin_array::get_bin_array_pda(&pool, expected_index);
            let matches = generated_pda.to_string() == expected_pda;
            println!(
                "Index {}: {} - {}",
                expected_index,
                if matches { "✓" } else { "✗" },
                expected_pda
            );
            if !matches {
                println!("  Generated: {}", generated_pda);
            }
        }
    }

    #[tokio::test]
    async fn test_mid_price_quick_estimate() {
        let account = load_data().unwrap();

        use rust_decimal::Decimal;

        // With decimals auto-scaling inside, XY * YX should be ~1 in atomic space.
        let quote_xy = account
            .mid_price_for_quick_estimate(&account.token_x_mint, &account.token_y_mint)
            .await
            .unwrap()
            .mid_price;
        let quote_yx = account
            .mid_price_for_quick_estimate(&account.token_y_mint, &account.token_x_mint)
            .await
            .unwrap()
            .mid_price;

        let product = quote_xy * quote_yx;
        let diff = (product - Decimal::ONE).abs();
        assert!(diff < Decimal::new(1, 9), "reciprocal check failed: {}", product);
    }

    const POOL_ADDRESS: &str = "8ztFxjFPfVUtEf4SLSapcFj8GW2dxyUA9no2bLPq7H7V";
    const ACCOUNT_DATA_BASE64: &str = "IQsxYrVlsQ0QJx4AWAKIEyBOAAAwVwUAtar//0tVAAD0AQAAAAAAAKlQAACJAgAAxgAAAAAAAACthZ1oAAAAAAAAAAAAAAAA/RQAA8gAAAAUAAAAECcAAMDwQqqsn4I3RswQ4QTTY1WRzy16NHGubwcnwI/bJ02FBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAFLIKeIqVbCJLfRzAXj0i57dYiuNT0BDGidCwPV101ZK/JBTOrKFnQ0+pYZB/CAnCaFTYy4e2m7WYU+0HudVPia3HxfclcAAADNXYQ0OAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2890xm4z7dNMN2joFKm10GFDBVccWYrFno7Rmd3nVw0AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2O7//wEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABULzqvBPVQwYswClugFtsyU938tEfqaHFwk5hbiY2f6AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACZ6kZcnXYJnUKGAndLzPGvshsD4aJ6oRLteCWio9S9RQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
    const ACCOUNT_DATA_JSON: &str = r#"
    {"parameters":{"type":{"defined":{"name":"StaticParameters"}},"data":{"base_factor":{"type":"u16","data":"10000"},"filter_period":{"type":"u16","data":"30"},"decay_period":{"type":"u16","data":"600"},"reduction_factor":{"type":"u16","data":"5000"},"variable_fee_control":{"type":"u32","data":"20000"},"max_volatility_accumulator":{"type":"u32","data":"350000"},"min_bin_id":{"type":"i32","data":"-21835"},"max_bin_id":{"type":"i32","data":"21835"},"protocol_share":{"type":"u16","data":"500"},"base_fee_power_factor":{"type":"u8","data":0},"_padding":{"type":{"array":["u8",5]},"data":[0,0,0,0,0]}}},"v_parameters":{"type":{"defined":{"name":"VariableParameters"}},"data":{"volatility_accumulator":{"type":"u32","data":"20649"},"volatility_reference":{"type":"u32","data":"649"},"index_reference":{"type":"i32","data":"198"},"_padding":{"type":{"array":["u8",4]},"data":[0,0,0,0]},"last_update_timestamp":{"type":"i64","data":"1755153837"},"_padding_1":{"type":{"array":["u8",8]},"data":[0,0,0,0,0,0,0,0]}}},"bump_seed":{"type":{"array":["u8",1]},"data":[253]},"bin_step_seed":{"type":{"array":["u8",2]},"data":[20,0]},"pair_type":{"type":"u8","data":3},"active_id":{"type":"i32","data":"200"},"bin_step":{"type":"u16","data":"20"},"status":{"type":"u8","data":0},"require_base_factor_seed":{"type":"u8","data":0},"base_factor_seed":{"type":{"array":["u8",2]},"data":[16,39]},"activation_type":{"type":"u8","data":0},"creator_pool_on_off_control":{"type":"u8","data":0},"token_x_mint":{"type":"pubkey","data":"Dz9mQ9NzkBcCsuGPFJ3r1bS4wgqKMHBPiVuniW8Mbonk"},"token_y_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"reserve_x":{"type":"pubkey","data":"64GTWbkiCgZt62EMccjFHRoT1MQAQviDioa63NCj37w8"},"reserve_y":{"type":"pubkey","data":"HJfR4mh9Yctrrh8pQQsrGsNdqV7KfpaaXGSdxGTwoeBK"},"protocol_fee":{"type":{"defined":{"name":"ProtocolFee"}},"data":{"amount_x":{"type":"u64","data":"375581015260"},"amount_y":{"type":"u64","data":"241399258573"}}},"_padding_1":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"reward_infos":{"type":{"array":[{"defined":{"name":"RewardInfo"}},2]},"data":[{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"},{"mint":"11111111111111111111111111111111","vault":"11111111111111111111111111111111","funder":"11111111111111111111111111111111","reward_duration":"0","reward_duration_end":"0","reward_rate":"0","last_update_time":"0","cumulative_seconds_with_empty_liquidity_reward":"0"}]},"oracle":{"type":"pubkey","data":"Fo3m9HQx8Rv4EMzmKWxe5yjCZMNcB5W5sKNv4pDzRFqe"},"bin_array_bitmap":{"type":{"array":["u64",16]},"data":["0","0","0","0","0","0","0","18441915018640359424","511","0","0","0","0","0","0","0"]},"last_updated_at":{"type":"i64","data":"0"},"_padding_2":{"type":{"array":["u8",32]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"pre_activation_swap_address":{"type":"pubkey","data":"11111111111111111111111111111111"},"base_key":{"type":"pubkey","data":"2RA1EnEVxWP8TQZhFt2nXuVcrQetFQUgYyGsUBTWUNpR"},"activation_point":{"type":"u64","data":"0"},"pre_activation_duration":{"type":"u64","data":"0"},"_padding_3":{"type":{"array":["u8",8]},"data":[0,0,0,0,0,0,0,0]},"_padding_4":{"type":"u64","data":"0"},"creator":{"type":"pubkey","data":"BMpa9wWzZepEgp7qxps9G72AnAwfFEQCWxboaNhop1BA"},"token_mint_x_program_flag":{"type":"u8","data":0},"token_mint_y_program_flag":{"type":"u8","data":0},"_reserved":{"type":{"array":["u8",22]},"data":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}}
    "#;
}
