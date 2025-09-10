use crate::util::alias::AResult;

#[derive(Debug, PartialEq)]
pub struct RaydiumCpmmIxData {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

impl RaydiumCpmmIxData {
    pub fn load_data(data_hex: &str) -> AResult<RaydiumCpmmIxData> {
        let data = hex::decode(data_hex)?;
        
        if data.len() < 24 {
            return Err(anyhow::anyhow!("Invalid data length: expected at least 24 bytes, got {}", data.len()));
        }
        
        let amount_in = u64::from_le_bytes(data[8..16].try_into()?);
        let minimum_amount_out = u64::from_le_bytes(data[16..24].try_into()?);
        
        Ok(RaydiumCpmmIxData {
            amount_in,
            minimum_amount_out,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_data() {
        let hex = "8fbe5adac41e33de223be0f0000000000d8212e800000000";
        let expected = RaydiumCpmmIxData {
            amount_in: 4041227042,
            minimum_amount_out: 3893527053,
        };
        let result = RaydiumCpmmIxData::load_data(hex).unwrap();
        assert_eq!(expected, result);
    }
}
