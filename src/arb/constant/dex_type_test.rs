#[cfg(test)]
mod tests {
    use super::super::dex_type::DexType;

    #[test]
    fn test_dex_type_roundtrip() {
        let test_cases = vec![
            (DexType::RaydiumV4, "RaydiumV4"),
            (DexType::RaydiumCp, "RaydiumCp"),
            (DexType::RaydiumClmm, "RaydiumClmm"),
            (DexType::Pump, "Pump"),
            (DexType::MeteoraDlmm, "MeteoraDlmm"),
            (DexType::MeteoraDamm, "MeteoraDamm"),
            (DexType::MeteoraDammV2, "MeteoraDammV2"),
            (DexType::OrcaWhirlpool, "OrcaWhirlpool"),
            (DexType::Solfi, "Solfi"),
            (DexType::Vertigo, "Vertigo"),
            (DexType::Unknown, "Unknown"),
        ];

        for (dex_type, expected_string) in test_cases {
            // Test to_db_string
            assert_eq!(
                dex_type.to_db_string(),
                expected_string,
                "to_db_string failed for {:?}",
                dex_type
            );

            // Test from_db_string
            assert_eq!(
                DexType::from_db_string(expected_string),
                dex_type,
                "from_db_string failed for {}",
                expected_string
            );

            // Test roundtrip
            let roundtrip = DexType::from_db_string(dex_type.to_db_string());
            assert_eq!(
                roundtrip, dex_type,
                "Roundtrip failed for {:?}",
                dex_type
            );
        }
    }

    #[test]
    fn test_dex_type_debug_format_matches_db() {
        // Verify that Debug format matches what we expect in DB
        let test_cases = vec![
            (DexType::RaydiumV4, "RaydiumV4"),
            (DexType::MeteoraDlmm, "MeteoraDlmm"),
            (DexType::OrcaWhirlpool, "OrcaWhirlpool"),
        ];

        for (dex_type, expected) in test_cases {
            let debug_str = format!("{:?}", dex_type);
            assert_eq!(
                debug_str, expected,
                "Debug format doesn't match expected for {:?}",
                dex_type
            );
            
            // Verify this matches to_db_string
            assert_eq!(
                dex_type.to_db_string(),
                expected,
                "to_db_string doesn't match Debug format for {:?}",
                dex_type
            );
        }
    }

    #[test]
    fn test_unknown_dex_type() {
        assert_eq!(DexType::from_db_string("InvalidDexType"), DexType::Unknown);
        assert_eq!(DexType::from_db_string(""), DexType::Unknown);
        assert_eq!(DexType::from_db_string("raydium_v4"), DexType::Unknown); // Wrong case
    }
}