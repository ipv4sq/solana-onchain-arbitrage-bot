use sea_orm::sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr};
use sea_orm::{DbErr, QueryResult, TryGetError, TryGetable, Value};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Copy, Default, Hash)]
#[serde(transparent)]
pub struct PubkeyType(pub Pubkey);

impl Deref for PubkeyType {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Pubkey> for PubkeyType {
    fn from(pubkey: Pubkey) -> Self {
        PubkeyType(pubkey)
    }
}

impl From<PubkeyType> for Pubkey {
    fn from(wrapper: PubkeyType) -> Self {
        wrapper.0
    }
}

impl From<PubkeyType> for Value {
    fn from(wrapper: PubkeyType) -> Self {
        Value::Bytes(Some(Box::new(wrapper.0.to_bytes().to_vec())))
    }
}

impl TryGetable for PubkeyType {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let bytes: Vec<u8> = Vec::<u8>::try_get(res, pre, col)?;
        if bytes.len() != 32 {
            return Err(TryGetError::DbErr(DbErr::Type(format!(
                "Invalid pubkey length: expected 32, got {}",
                bytes.len()
            ))));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(PubkeyType(Pubkey::from(array)))
    }

    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let bytes: Vec<u8> = Vec::<u8>::try_get_by(res, index)?;
        if bytes.len() != 32 {
            return Err(TryGetError::DbErr(DbErr::Type(format!(
                "Invalid pubkey length: expected 32, got {}",
                bytes.len()
            ))));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(PubkeyType(Pubkey::from(array)))
    }
}

impl ValueType for PubkeyType {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Bytes(Some(bytes)) => {
                if bytes.len() != 32 {
                    return Err(ValueTypeErr);
                }
                let mut array = [0u8; 32];
                array.copy_from_slice(&bytes);
                Ok(PubkeyType(Pubkey::from(array)))
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "PubkeyType".to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::Bytes
    }

    fn column_type() -> ColumnType {
        ColumnType::Binary(32)
    }
}

impl Nullable for PubkeyType {
    fn null() -> Value {
        Value::Bytes(None)
    }
}

impl sea_orm::TryFromU64 for PubkeyType {
    fn try_from_u64(_n: u64) -> Result<Self, DbErr> {
        Err(DbErr::Type(
            "PubkeyType cannot be created from u64".to_string(),
        ))
    }
}

impl AsRef<Pubkey> for PubkeyType {
    fn as_ref(&self) -> &Pubkey {
        &self.0
    }
}

impl std::fmt::Display for PubkeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
