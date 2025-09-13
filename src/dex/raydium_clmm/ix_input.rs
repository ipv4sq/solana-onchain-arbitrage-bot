use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RaydiumClmmIxData {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
}

impl RaydiumClmmIxData {
    pub fn to_bytes_with_discriminator(&self, use_v2: bool) -> Vec<u8> {
        let discriminator = if use_v2 {
            [43, 4, 237, 11, 26, 201, 30, 98] // swap_v2
        } else {
            [248, 198, 158, 145, 225, 117, 135, 200] // swap (deprecated)
        };

        let mut data = discriminator.to_vec();
        data.extend(borsh::to_vec(self).expect("Failed to serialize RaydiumClmmIxData"));
        data
    }

    pub fn to_hex(&self, use_v2: bool) -> String {
        hex::encode(self.to_bytes_with_discriminator(use_v2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_swap_instruction() {
        let hex_data = "2b04ed0b1ac91e62099c1d5f01000000d3ec8c9000000000cabc92b9fd7d257e000000000000000001";
        let bytes = hex::decode(hex_data).unwrap();

        let ix_data = RaydiumClmmIxData::try_from_slice(&bytes[8..]).unwrap();

        assert_eq!(ix_data.amount, 5890743305);
        assert_eq!(ix_data.other_amount_threshold, 2425154771);
        assert_eq!(ix_data.sqrt_price_limit_x64, 9089809951610813642);
        assert_eq!(ix_data.is_base_input, true);
    }

    #[test]
    fn test_serialize_swap_instruction() {
        let ix_data = RaydiumClmmIxData {
            amount: 1000000000,
            other_amount_threshold: 900000000,
            sqrt_price_limit_x64: 0,
            is_base_input: true,
        };

        // Test swap_v2 discriminator
        let bytes_v2 = ix_data.to_bytes_with_discriminator(true);
        assert_eq!(&bytes_v2[..8], &[43, 4, 237, 11, 26, 201, 30, 98]);

        // Test swap (deprecated) discriminator
        let bytes_v1 = ix_data.to_bytes_with_discriminator(false);
        assert_eq!(&bytes_v1[..8], &[248, 198, 158, 145, 225, 117, 135, 200]);

        // Verify we can deserialize the serialized data (without discriminator)
        let deserialized = RaydiumClmmIxData::try_from_slice(&bytes_v2[8..]).unwrap();
        assert_eq!(deserialized.amount, ix_data.amount);
        assert_eq!(deserialized.other_amount_threshold, ix_data.other_amount_threshold);
        assert_eq!(deserialized.sqrt_price_limit_x64, ix_data.sqrt_price_limit_x64);
        assert_eq!(deserialized.is_base_input, ix_data.is_base_input);
    }
}
