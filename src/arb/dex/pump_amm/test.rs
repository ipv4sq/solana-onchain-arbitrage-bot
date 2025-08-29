#[cfg(test)]
mod tests {
    use crate::arb::dex::interface::PoolDataLoader;
    use crate::arb::dex::pump_amm::pool_data::PumpAmmPoolData;
    use crate::arb::util::traits::pubkey::ToPubkey;

    #[test]
    fn test_get_creator_vault_authority() {
        let expected = "wYGQPcWwnpSV1eqLQwatefR9ihwUtWoybNZcg8uvQWU";
        let coin_creator = "GeUnv1jmtviRbR7Gu1JnXSGkUMUgFVBHuEVQVpTaUX1W".to_pubkey();
        assert_eq!(
            PumpAmmPoolData::get_creator_vault_authority(&coin_creator),
            expected.to_pubkey()
        )
    }

    #[test]
    fn test_parse_pump_amm_account_data() {
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::Value;

        let data = general_purpose::STANDARD
            .decode(ACCOUNT_DATA_BASE64)
            .unwrap();
        let account = PumpAmmPoolData::load_data(&data).expect("Failed to parse account data");

        let json: Value = serde_json::from_str(ACCOUNT_DATA_JSON).expect("Failed to parse JSON");

        assert_eq!(
            account.pool_bump,
            json["pool_bump"]["data"].as_u64().unwrap() as u8
        );
        assert_eq!(
            account.index,
            json["index"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );
        assert_eq!(
            account.lp_supply,
            json["lp_supply"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );

        assert_eq!(
            account.creator.to_string(),
            json["creator"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.base_mint.to_string(),
            json["base_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.quote_mint.to_string(),
            json["quote_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.lp_mint.to_string(),
            json["lp_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.pool_base_token_account.to_string(),
            json["pool_base_token_account"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.pool_quote_token_account.to_string(),
            json["pool_quote_token_account"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.coin_creator.to_string(),
            json["coin_creator"]["data"].as_str().unwrap()
        );
    }

    static POOL_ADDRESS: &str = "GUXAutvXh2Cvv2avGkbY8CfcsN9v2Uiwr8VCCqpn9HiU";
    static ACCOUNT_DATA_BASE64: &str = "8ZptBBGxbbz/AAAs98JusIgYjN/c+OImcGGKLbklh9KFdMPdkckEOMtRRiRJ+U7BLbz0edwLteAk1ZOEusHMAqkS9JpDqKYXM3n5BpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEIy+Mec8oHNW4ats6+SDWH6uwUdvnAkxqo7K1PvyTB1HQFfSZ1kp+pDQsMXajk+o6uiopINyd6LPEjQWvMErx5qIq9uP3CEJTTRxA+SM03rI36/NlFGPup8uXrVy0f5uRvp2tZ0AMAAOh5N8loi43Vzzi7J8beMK5OsQsmLpxQq4JX1IbnINydAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    static ACCOUNT_DATA_JSON: &str = r#"
{"pool_bump":{"type":"u8","data":255},"index":{"type":"u16","data":"0"},"creator":{"type":"pubkey","data":"42Y4PfN8eHhZJbp5vfqc59sMhw85g1QSdQ4WNYQgkgcM"},"base_mint":{"type":"pubkey","data":"3Sf6oKCeEqCuco4aYtKHHDTBYLAWHiL47QjvkW1UYDEG"},"quote_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"lp_mint":{"type":"pubkey","data":"bLafbwgNhmVvqLuxQWVrFY7QiwTU5FWNtd21WzAwxAo"},"pool_base_token_account":{"type":"pubkey","data":"8ou9YbVwLfMRXFq1ejqJcdXX9GM1bSAZUKrYzox3bSvc"},"pool_quote_token_account":{"type":"pubkey","data":"CLvCQE8qTSPmHJrxDqyrwP5hCvvuoha8tu7jY7bhxHFZ"},"lp_supply":{"type":"u64","data":"4193388308335"},"coin_creator":{"type":"pubkey","data":"GeUnv1jmtviRbR7Gu1JnXSGkUMUgFVBHuEVQVpTaUX1W"}}
"#;
}
