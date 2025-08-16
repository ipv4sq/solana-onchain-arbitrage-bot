use crate::arb::constant::mint::MintPair;
use crate::arb::tx::constants::DexType;
use anyhow::anyhow;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Debug)]
pub struct SmbInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<Pubkey>,
    pub data: SmbIxParameter,
}

#[derive(Debug)]
pub struct SmbIxParameter {
    pub instruction_discriminator: u8,
    pub minimum_profit: u64,
    pub compute_unit_limit: u32,
    pub no_failure_mode: bool,
    pub reserved: u16,
    pub use_flashloan: bool,
    pub raw_data: Vec<u8>,
}

impl SmbIxParameter {
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() >= 17 {
            Ok(Self {
                instruction_discriminator: data[0],
                minimum_profit: u64::from_le_bytes(data[1..9].try_into()?),
                compute_unit_limit: u32::from_le_bytes(data[9..13].try_into()?),
                no_failure_mode: data[13] != 0,
                reserved: u16::from_le_bytes(data[14..16].try_into()?),
                use_flashloan: data[16] != 0,
                raw_data: data.to_vec(),
            })
        } else {
            // Return default/empty parameters for invalid data
            Err(anyhow!(
                "Invalid parameter data: expected 17 bytes, got {} bytes. Hex: {}",
                data.len(),
                data.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()
            ))
        }
    }

    pub fn to_hex(&self) -> String {
        self.raw_data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.raw_data).into_string()
    }

    pub fn is_arbitrage_instruction(&self) -> bool {
        self.instruction_discriminator == 28
    }
}

#[derive(Debug, Clone)]
pub struct SwapInstruction {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub mints: MintPair,
}

#[derive(Debug, Clone)]
pub struct LitePool {
    pub dex_type: DexType,
    pub pool_address: Pubkey,
    pub mints: MintPair,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smb_ix_parameter_parsing() {
        // Test data from the actual transaction
        let data = vec![
            0x1c, // discriminator = 28
            0xa1, 0xdd, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, // minimum_profit = 253345
            0xa0, 0xd9, 0x08, 0x00, // compute_unit_limit = 580000
            0x00, // no_failure_mode = false
            0x00, 0x00, // reserved = 0
            0x01, // use_flashloan = true
        ];

        let params = SmbIxParameter::from_bytes(&data).unwrap();

        assert_eq!(params.instruction_discriminator, 28);
        assert_eq!(params.minimum_profit, 253345);
        assert_eq!(params.compute_unit_limit, 580000);
        assert_eq!(params.no_failure_mode, false);
        assert_eq!(params.reserved, 0);
        assert_eq!(params.use_flashloan, true);
        assert!(params.is_arbitrage_instruction());

        // Test hex and base58 conversion
        assert_eq!(params.to_hex(), "1ca1dd030000000000a0d9080000000001");
        assert_eq!(params.to_base58(), "Gc881PaDcBFZens2MnZcD1z");
    }

    #[test]
    fn test_smb_ix_parameter_invalid_data() {
        // Test with insufficient data
        let short_data = vec![0x1c, 0xa1];
        let params = SmbIxParameter::from_bytes(&short_data).unwrap();

        assert_eq!(params.instruction_discriminator, 28);
        assert_eq!(params.minimum_profit, 0);
        assert_eq!(params.compute_unit_limit, 0);
        assert_eq!(params.raw_data, short_data);
    }
}
