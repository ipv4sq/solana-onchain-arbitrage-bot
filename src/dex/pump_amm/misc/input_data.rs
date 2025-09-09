use hex;

#[derive(Debug, PartialEq)]
pub struct PumpAmmIxData {
    // exact in
    // base -> quote
    pub base_amount_in: Option<u64>,
    pub min_quote_amount_out: Option<u64>,
    // quote -> base
    pub quote_amount_in: Option<u64>,
    pub min_base_amount_out: Option<u64>,

    // exact out
    // quote -> base
    pub base_amount_out: Option<u64>,
    pub max_quote_amount_in: Option<u64>,
    // base -> quote
    pub quote_amount_out: Option<u64>,
    pub max_base_amount_in: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpSwapDirection {
    Buy,  // base -> quote (exact in)
    Sell, // quote -> base (exact out)
}

impl PumpAmmIxData {
    pub fn load_ix_data(data_hex: &str) -> PumpAmmIxData {
        let decoded = hex::decode(data_hex).expect("Failed to decode hex");

        // Skip the first 8 bytes (instruction discriminator)
        let discriminator = &decoded[0..8];

        // Parse all 8 possible u64 fields (8 bytes each) after the discriminator
        // The instruction data should have: 8 (discriminator) + 8*8 (fields) = 72 bytes total
        // But some instructions might have fewer fields, so we'll parse what's available

        let parse_u64_at = |offset: usize| -> u64 {
            if decoded.len() >= offset + 8 {
                let bytes = decoded[offset..offset + 8]
                    .try_into()
                    .expect("Failed to parse u64");
                u64::from_le_bytes(bytes)
            } else {
                0
            }
        };

        // Map fields to struct based on discriminator
        // The actual mapping depends on the swap type
        if discriminator == [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea] {
            // Sell instruction (exact out) - only 2 fields are used
            let field1 = parse_u64_at(8);
            let field2 = parse_u64_at(16);

            PumpAmmIxData {
                base_amount_in: None,
                min_quote_amount_out: None,
                quote_amount_in: None,
                min_base_amount_out: None,
                base_amount_out: Some(field1),
                max_quote_amount_in: Some(field2),
                quote_amount_out: None,
                max_base_amount_in: None,
            }
        } else if discriminator == [0x33, 0xe6, 0x85, 0xa4, 0x01, 0x7f, 0x83, 0xad] {
            // Buy instruction (exact in) - only 2 fields are used
            let field1 = parse_u64_at(8);
            let field2 = parse_u64_at(16);

            PumpAmmIxData {
                base_amount_in: Some(field1),
                min_quote_amount_out: Some(field2),
                quote_amount_in: None,
                min_base_amount_out: None,
                base_amount_out: None,
                max_quote_amount_in: None,
                quote_amount_out: None,
                max_base_amount_in: None,
            }
        } else {
            // Unknown discriminator - parse all 8 fields in case they're present
            let field1 = parse_u64_at(8);
            let field2 = parse_u64_at(16);
            let field3 = parse_u64_at(24);
            let field4 = parse_u64_at(32);
            let field5 = parse_u64_at(40);
            let field6 = parse_u64_at(48);
            let field7 = parse_u64_at(56);
            let field8 = parse_u64_at(64);

            // For unknown discriminators, only populate fields that have non-zero values
            PumpAmmIxData {
                base_amount_in: if field1 != 0 { Some(field1) } else { None },
                min_quote_amount_out: if field2 != 0 { Some(field2) } else { None },
                quote_amount_in: if field3 != 0 { Some(field3) } else { None },
                min_base_amount_out: if field4 != 0 { Some(field4) } else { None },
                base_amount_out: if field5 != 0 { Some(field5) } else { None },
                max_quote_amount_in: if field6 != 0 { Some(field6) } else { None },
                quote_amount_out: if field7 != 0 { Some(field7) } else { None },
                max_base_amount_in: if field8 != 0 { Some(field8) } else { None },
            }
        }
    }

    pub fn to_hex(&self, direction: PumpSwapDirection) -> String {
        let mut data = Vec::new();
        
        match direction {
            PumpSwapDirection::Buy => {
                // Buy instruction (base -> quote, exact in)
                let discriminator = [0x33, 0xe6, 0x85, 0xa4, 0x01, 0x7f, 0x83, 0xad];
                data.extend_from_slice(&discriminator);
                
                // Add base_amount_in and min_quote_amount_out
                let base_amount_in = self.base_amount_in.unwrap_or(0);
                let min_quote_amount_out = self.min_quote_amount_out.unwrap_or(0);
                
                data.extend_from_slice(&base_amount_in.to_le_bytes());
                data.extend_from_slice(&min_quote_amount_out.to_le_bytes());
            }
            PumpSwapDirection::Sell => {
                // Sell instruction (quote -> base, exact out)
                let discriminator = [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea];
                data.extend_from_slice(&discriminator);
                
                // Add base_amount_out and max_quote_amount_in
                let base_amount_out = self.base_amount_out.unwrap_or(0);
                let max_quote_amount_in = self.max_quote_amount_in.unwrap_or(0);
                
                data.extend_from_slice(&base_amount_out.to_le_bytes());
                data.extend_from_slice(&max_quote_amount_in.to_le_bytes());
            }
        }
        
        hex::encode(data)
    }
    
    pub fn get_discriminator(direction: PumpSwapDirection) -> [u8; 8] {
        match direction {
            PumpSwapDirection::Buy => [0x33, 0xe6, 0x85, 0xa4, 0x01, 0x7f, 0x83, 0xad],
            PumpSwapDirection::Sell => [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea],
        }
    }
    
    pub fn detect_direction(&self) -> Option<PumpSwapDirection> {
        // Detect direction based on which fields are populated
        if self.base_amount_in.is_some() && self.min_quote_amount_out.is_some() {
            Some(PumpSwapDirection::Buy)
        } else if self.base_amount_out.is_some() && self.max_quote_amount_in.is_some() {
            Some(PumpSwapDirection::Sell)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dex::pump_amm::misc::input_data::{PumpAmmIxData, PumpSwapDirection};

    #[test]
    fn test_input_data() {
        let hex = "66063d1201daebea1f2ad632be01000017d0a0b800000000";
        let expected = PumpAmmIxData {
            base_amount_in: None,
            min_quote_amount_out: None,
            quote_amount_in: None,
            min_base_amount_out: None,
            base_amount_out: Some(1916408310303),
            max_quote_amount_in: Some(3097546775),
            quote_amount_out: None,
            max_base_amount_in: None,
        };
        let actual = PumpAmmIxData::load_ix_data(hex);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_input_data_2() {
        let hex = "33e685a4017f83ad81608110420000000000000000000000";
        let expected = PumpAmmIxData {
            base_amount_in: Some(283744755841),
            min_quote_amount_out: Some(0),
            quote_amount_in: None,
            min_base_amount_out: None,
            base_amount_out: None,
            max_quote_amount_in: None,
            quote_amount_out: None,
            max_base_amount_in: None,
        };
        let actual = PumpAmmIxData::load_ix_data(hex);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_input_data_3() {
        let hex = "66063d1201daebea72cfa72f0800000082ae9c5200000000";
        let expected = PumpAmmIxData {
            base_amount_in: None,
            min_quote_amount_out: None,
            quote_amount_in: None,
            min_base_amount_out: None,
            base_amount_out: Some(35159265138),
            max_quote_amount_in: Some(1386000002),
            quote_amount_out: None,
            max_base_amount_in: None,
        };
        let actual = PumpAmmIxData::load_ix_data(hex);
        assert_eq!(expected, actual);
    }
    
    #[test]
    fn test_round_trip_sell_direction() {
        // Test Sell direction (exact out)
        let original_hex = "66063d1201daebea1f2ad632be01000017d0a0b800000000";
        
        // Parse the hex
        let parsed = PumpAmmIxData::load_ix_data(original_hex);
        
        // Verify it's detected as Sell direction
        assert_eq!(parsed.detect_direction(), Some(PumpSwapDirection::Sell));
        
        // Convert back to hex
        let restored_hex = parsed.to_hex(PumpSwapDirection::Sell);
        
        // Should match original
        assert_eq!(restored_hex, original_hex);
    }
    
    #[test]
    fn test_round_trip_buy_direction() {
        // Test Buy direction (exact in)
        let original_hex = "33e685a4017f83ad81608110420000000000000000000000";
        
        // Parse the hex
        let parsed = PumpAmmIxData::load_ix_data(original_hex);
        
        // Verify it's detected as Buy direction
        assert_eq!(parsed.detect_direction(), Some(PumpSwapDirection::Buy));
        
        // Convert back to hex
        let restored_hex = parsed.to_hex(PumpSwapDirection::Buy);
        
        // Should match original
        assert_eq!(restored_hex, original_hex);
    }
    
    #[test]
    fn test_create_and_encode_buy() {
        // Create a new Buy instruction data
        let ix_data = PumpAmmIxData {
            base_amount_in: Some(1000000),
            min_quote_amount_out: Some(500000),
            quote_amount_in: None,
            min_base_amount_out: None,
            base_amount_out: None,
            max_quote_amount_in: None,
            quote_amount_out: None,
            max_base_amount_in: None,
        };
        
        // Encode to hex
        let hex = ix_data.to_hex(PumpSwapDirection::Buy);
        
        // Parse it back
        let parsed = PumpAmmIxData::load_ix_data(&hex);
        
        // Should match original
        assert_eq!(parsed.base_amount_in, Some(1000000));
        assert_eq!(parsed.min_quote_amount_out, Some(500000));
        assert_eq!(parsed.detect_direction(), Some(PumpSwapDirection::Buy));
    }
    
    #[test]
    fn test_create_and_encode_sell() {
        // Create a new Sell instruction data
        let ix_data = PumpAmmIxData {
            base_amount_in: None,
            min_quote_amount_out: None,
            quote_amount_in: None,
            min_base_amount_out: None,
            base_amount_out: Some(2000000),
            max_quote_amount_in: Some(1500000),
            quote_amount_out: None,
            max_base_amount_in: None,
        };
        
        // Encode to hex
        let hex = ix_data.to_hex(PumpSwapDirection::Sell);
        
        // Parse it back
        let parsed = PumpAmmIxData::load_ix_data(&hex);
        
        // Should match original
        assert_eq!(parsed.base_amount_out, Some(2000000));
        assert_eq!(parsed.max_quote_amount_in, Some(1500000));
        assert_eq!(parsed.detect_direction(), Some(PumpSwapDirection::Sell));
    }
}
