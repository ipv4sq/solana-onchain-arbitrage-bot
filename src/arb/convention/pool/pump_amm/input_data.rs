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

impl PumpAmmIxData {
    pub fn load_ix_data(data_hex: &str) -> PumpAmmIxData {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::arb::convention::pool::pump_amm::input_data::PumpAmmIxData;

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
}
