use crate::arb::database::columns::PubkeyType;
use solana_program::pubkey::Pubkey;

pub trait ToOrm {
    fn to_orm(&self) -> PubkeyType;
}

impl ToOrm for Pubkey {
    fn to_orm(&self) -> PubkeyType {
        PubkeyType(*self)
    }
}