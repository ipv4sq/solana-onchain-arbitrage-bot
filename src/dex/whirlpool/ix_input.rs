use crate::util::alias::AResult;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum AccountsType {
    TransferHookA,
    TransferHookB,
    TransferHookReward,
    TransferHookInput,
    TransferHookIntermediate,
    TransferHookOutput,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct RemainingAccountsInfo {
    pub slices: Vec<RemainingAccountsSlice>,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct WhirlpoolIxData {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
    pub remaining_accounts_info: Option<RemainingAccountsInfo>,
}

impl WhirlpoolIxData {
    const SWAP_V2_DISCRIMINATOR: [u8; 8] = [0x2b, 0x04, 0xed, 0x0b, 0x1a, 0xc9, 0x1e, 0x62];

    pub fn load(data: &[u8]) -> AResult<WhirlpoolIxData> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("Data too short for SwapV2 instruction"));
        }

        if data[0..8] != Self::SWAP_V2_DISCRIMINATOR {
            return Err(anyhow::anyhow!("Invalid SwapV2 discriminator"));
        }

        Ok(WhirlpoolIxData::try_from_slice(&data[8..])?)
    }

    pub fn to_hex(&self) -> String {
        let mut bytes = Self::SWAP_V2_DISCRIMINATOR.to_vec();
        bytes.extend(borsh::to_vec(self).unwrap_or_default());
        hex::encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let hex_data = "2b04ed0b1ac91e626086553aa500000001000000000000000fc21313e2b28f140000000000000000010100";
        let data = hex::decode(hex_data).unwrap();

        let ix_data = WhirlpoolIxData::load(&data).unwrap();

        assert_eq!(ix_data.amount, 709648287328);
        assert_eq!(ix_data.other_amount_threshold, 1);
        assert_eq!(ix_data.sqrt_price_limit, 1481599486480597519);
        assert_eq!(ix_data.amount_specified_is_input, true);
        assert_eq!(ix_data.a_to_b, true);
        assert_eq!(ix_data.remaining_accounts_info, None);
    }

    #[test]
    fn test_to_hex() {
        let ix_data = WhirlpoolIxData {
            amount: 709648287328,
            other_amount_threshold: 1,
            sqrt_price_limit: 1481599486480597519,
            amount_specified_is_input: true,
            a_to_b: true,
            remaining_accounts_info: None,
        };

        let hex = ix_data.to_hex();

        let expected = "2b04ed0b1ac91e626086553aa500000001000000000000000fc21313e2b28f140000000000000000010100";
        assert_eq!(hex, expected);
    }
}
