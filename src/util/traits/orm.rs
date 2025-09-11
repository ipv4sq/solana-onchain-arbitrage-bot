use crate::database::columns::{PubkeyType, PubkeyTypeString};
use solana_program::pubkey::Pubkey;

pub trait ToOrm {
    fn to_orm(&self) -> PubkeyType;
}

impl ToOrm for Pubkey {
    fn to_orm(&self) -> PubkeyType {
        PubkeyType(*self)
    }
}

pub trait ToOrmString {
    fn to_orm(&self) -> PubkeyTypeString;
}

impl ToOrmString for Pubkey {
    fn to_orm(&self) -> PubkeyTypeString {
        PubkeyTypeString(*self)
    }
}
