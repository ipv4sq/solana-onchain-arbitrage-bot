use crate::arb::chain::ix;
use crate::arb::constant::pool_owner::PoolOwnerPrograms;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_transaction_status::{UiInstruction, UiPartiallyDecodedInstruction};

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct MeteoraDammV2InputData {
    pub discriminator: [u8; 8],
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

impl MeteoraDammV2InputData {
    pub fn load_from_hex(hex: &str) -> Result<MeteoraDammV2InputData> {
        let bytes = hex::decode(hex).map_err(|e| anyhow::anyhow!("Failed to decode hex: {}", e))?;

        if bytes.len() < 24 {
            return Err(anyhow::anyhow!(
                "Invalid instruction data length: expected at least 24 bytes, got {}",
                bytes.len()
            ));
        }

        let mut discriminator = [0u8; 8];
        discriminator.copy_from_slice(&bytes[0..8]);

        let mut data_slice = &bytes[8..];
        let amount_in = u64::deserialize(&mut data_slice)?;
        let minimum_amount_out = u64::deserialize(&mut data_slice)?;

        Ok(MeteoraDammV2InputData {
            discriminator,
            amount_in,
            minimum_amount_out,
        })
    }
}
pub fn is_meteora_damm_v2_swap(ix: &UiInstruction) -> Option<&UiPartiallyDecodedInstruction> {
    // METEORA_DAMM_V2 swap instructions have exactly 14 accounts
    ix::is_program_ix(ix, PoolOwnerPrograms::METEORA_DAMM_V2, Some(14))
        .filter(|decoded| decoded.accounts.len() == 14)
}
#[cfg(test)]
mod tests {
    use crate::arb::pool::meteora_damm_v2::input_data::MeteoraDammV2InputData;

    #[test]
    fn test_load_from_hex() {
        let hex = "f8c69e91e17587c8373b4ec0000000000000000000000000";
        let json = r#"
        
        "#;
        let result = MeteoraDammV2InputData::load_from_hex(hex).unwrap();

        // Check discriminator
        assert_eq!(
            result.discriminator,
            [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]
        );

        // Check amount_in (3226352439 in little-endian)
        assert_eq!(result.amount_in, 3226352439);

        // Check minimum_amount_out
        assert_eq!(result.minimum_amount_out, 0);
    }
}
