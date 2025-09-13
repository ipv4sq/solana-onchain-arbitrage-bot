use crate::util::alias::AResult;

pub struct WhirlpoolIxData {}

impl WhirlpoolIxData {
    pub fn load(data: &[u8]) -> AResult<WhirlpoolIxData> {
        todo!()
    }

    pub fn to_hex(&self) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {}

    #[test]
    fn test_to_hex() {}
}
