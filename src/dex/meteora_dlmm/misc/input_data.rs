use hex;

#[derive(Debug, PartialEq)]
pub struct MeteoraDlmmIxData {
    pub amount_in: u64,
    pub min_amount_out: u64,
}

impl MeteoraDlmmIxData {
    pub fn load_ix_data(data: &str) -> MeteoraDlmmIxData {
        let decoded = hex::decode(data).expect("Failed to decode hex");

        // Skip the first 8 bytes (instruction discriminator)
        let amount_in = u64::from_le_bytes(
            decoded[8..16]
                .try_into()
                .expect("Failed to parse amount_in"),
        );
        let min_amount_out = u64::from_le_bytes(
            decoded[16..24]
                .try_into()
                .expect("Failed to parse min_amount_out"),
        );

        MeteoraDlmmIxData {
            amount_in,
            min_amount_out,
        }
    }

    pub fn to_hex(&self) -> String {
        let discriminator = [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8];
        let mut data = Vec::with_capacity(24);
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&self.amount_in.to_le_bytes());
        data.extend_from_slice(&self.min_amount_out.to_le_bytes());
        hex::encode(data)
    }
}

pub fn is_meteora_dlmm_swap(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }
    let discriminator = &data[0..8];
    discriminator == [0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]
}

#[cfg(test)]
mod tests {
    use crate::dex::meteora_dlmm::misc::input_data::MeteoraDlmmIxData;

    static HEX_DATA: &str = "f8c69e91e17587c8ceaf31fc11ee01000000000000000000";

    #[test]
    fn test_load_ix_data() {
        let result = MeteoraDlmmIxData::load_ix_data(HEX_DATA);

        assert_eq!(result.amount_in, 543235989680078);
        assert_eq!(result.min_amount_out, 0);
    }

    #[test]
    fn test_load_data_2() {
        let result =
            MeteoraDlmmIxData::load_ix_data("f8c69e91e17587c8f1498cc602000000862ed9e30c000000");
        assert_eq!(result.amount_in, 11921017329);
        assert_eq!(result.min_amount_out, 55362268806);
    }

    #[test]
    fn test_to_hex() {
        let data = MeteoraDlmmIxData {
            amount_in: 543235989680078,
            min_amount_out: 0,
        };
        assert_eq!(data.to_hex(), HEX_DATA);

        let data2 = MeteoraDlmmIxData {
            amount_in: 11921017329,
            min_amount_out: 55362268806,
        };
        assert_eq!(
            data2.to_hex(),
            "f8c69e91e17587c8f1498cc602000000862ed9e30c000000"
        );
    }
}
