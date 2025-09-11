use sea_orm::sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr};
use sea_orm::{DbErr, QueryResult, TryGetError, TryGetable, Value};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Copy, Default, Hash)]
#[serde(transparent)]
pub struct PubkeyTypeString(pub Pubkey);

impl Deref for PubkeyTypeString {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Pubkey> for PubkeyTypeString {
    fn from(pubkey: Pubkey) -> Self {
        PubkeyTypeString(pubkey)
    }
}

impl From<PubkeyTypeString> for Pubkey {
    fn from(wrapper: PubkeyTypeString) -> Self {
        wrapper.0
    }
}

impl From<PubkeyTypeString> for Value {
    fn from(wrapper: PubkeyTypeString) -> Self {
        Value::String(Some(Box::new(wrapper.0.to_string())))
    }
}

impl TryGetable for PubkeyTypeString {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let val: String = String::try_get_by(res, index)?;
        Pubkey::from_str(&val)
            .map(PubkeyTypeString)
            .map_err(|e| TryGetError::DbErr(DbErr::Type(format!("Invalid pubkey string: {}", e))))
    }

    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let val: String = String::try_get(res, pre, col)?;
        Pubkey::from_str(&val)
            .map(PubkeyTypeString)
            .map_err(|e| TryGetError::DbErr(DbErr::Type(format!("Invalid pubkey string: {}", e))))
    }
}

impl ValueType for PubkeyTypeString {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(s)) => Pubkey::from_str(&s)
                .map(PubkeyTypeString)
                .map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "PubkeyTypeString".to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::String(sea_orm::sea_query::StringLen::N(44))
    }
}

impl Nullable for PubkeyTypeString {
    fn null() -> Value {
        Value::String(None)
    }
}

impl sea_orm::TryFromU64 for PubkeyTypeString {
    fn try_from_u64(_n: u64) -> Result<Self, DbErr> {
        Err(DbErr::Type(
            "PubkeyTypeString cannot be created from u64".to_string(),
        ))
    }
}

impl AsRef<Pubkey> for PubkeyTypeString {
    fn as_ref(&self) -> &Pubkey {
        &self.0
    }
}

impl std::fmt::Display for PubkeyTypeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
