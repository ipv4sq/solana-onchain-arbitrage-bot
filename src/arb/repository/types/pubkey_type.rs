use sea_orm::{DbErr, QueryResult, TryGetError, TryGetable, Value};
use sea_orm::sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr};
use solana_program::pubkey::Pubkey;
use std::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PubkeyWrapper(pub Pubkey);

impl Deref for PubkeyWrapper {
    type Target = Pubkey;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Pubkey> for PubkeyWrapper {
    fn from(pubkey: Pubkey) -> Self {
        PubkeyWrapper(pubkey)
    }
}

impl From<PubkeyWrapper> for Pubkey {
    fn from(wrapper: PubkeyWrapper) -> Self {
        wrapper.0
    }
}

impl From<PubkeyWrapper> for Value {
    fn from(wrapper: PubkeyWrapper) -> Self {
        Value::Bytes(Some(Box::new(wrapper.0.to_bytes().to_vec())))
    }
}

impl TryGetable for PubkeyWrapper {
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
        Ok(PubkeyWrapper(Pubkey::from(array)))
    }
}

impl ValueType for PubkeyWrapper {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Bytes(Some(bytes)) => {
                if bytes.len() != 32 {
                    return Err(ValueTypeErr);
                }
                let mut array = [0u8; 32];
                array.copy_from_slice(&bytes);
                Ok(PubkeyWrapper(Pubkey::from(array)))
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "PubkeyWrapper".to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::Bytes
    }

    fn column_type() -> ColumnType {
        ColumnType::Binary(sea_orm::sea_query::BlobSize::Blob(Some(32)))
    }
}

impl Nullable for PubkeyWrapper {
    fn null() -> Value {
        Value::Bytes(None)
    }
}

impl sea_orm::TryFromU64 for PubkeyWrapper {
    fn try_from_u64(_n: u64) -> Result<Self, DbErr> {
        Err(DbErr::Type("PubkeyWrapper cannot be created from u64".to_string()))
    }
}