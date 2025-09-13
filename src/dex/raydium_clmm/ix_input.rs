use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RaydiumClmmIxData {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
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
}
