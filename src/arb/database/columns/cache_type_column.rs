use crate::arb::util::structs::cache_type::CacheType;
use sea_orm::entity::prelude::*;
use sea_orm::{DbErr, TryGetError, TryGetable};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheTypeColumn(pub CacheType);

impl From<CacheType> for CacheTypeColumn {
    fn from(cache_type: CacheType) -> Self {
        CacheTypeColumn(cache_type)
    }
}

impl From<CacheTypeColumn> for CacheType {
    fn from(column: CacheTypeColumn) -> Self {
        column.0
    }
}

impl From<CacheTypeColumn> for Value {
    fn from(column: CacheTypeColumn) -> Self {
        Value::String(Some(Box::new(column.0.as_str().to_string())))
    }
}

impl TryGetable for CacheTypeColumn {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let column = format!("{}{}", pre, col);
        let value = res.try_get::<String>("", &column).map_err(TryGetError::DbErr)?;
        let cache_type = match value.as_str() {
            "mint_record" => CacheType::MintRecord,
            // Support legacy values for backward compatibility
            "mint_info" => CacheType::MintRecord,
            custom => CacheType::Custom(custom.to_string()),
        };
        Ok(CacheTypeColumn(cache_type))
    }
    
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let value = res.try_get_by::<String, I>(index).map_err(TryGetError::DbErr)?;
        let cache_type = match value.as_str() {
            "mint_record" => CacheType::MintRecord,
            // Support legacy values for backward compatibility
            "mint_info" => CacheType::MintRecord,
            custom => CacheType::Custom(custom.to_string()),
        };
        Ok(CacheTypeColumn(cache_type))
    }
}

impl sea_orm::TryFromU64 for CacheTypeColumn {
    fn try_from_u64(_n: u64) -> Result<Self, DbErr> {
        Err(DbErr::ConvertFromU64(
            "CacheTypeColumn cannot be created from u64",
        ))
    }
}

impl fmt::Display for CacheTypeColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl sea_orm::sea_query::ValueType for CacheTypeColumn {
    fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(s)) => {
                let cache_type = match s.as_str() {
                    "mint_record" => CacheType::MintRecord,
                    // Support legacy values for backward compatibility
                    "mint_info" => CacheType::MintRecord,
                    custom => CacheType::Custom(custom.to_string()),
                };
                Ok(CacheTypeColumn(cache_type))
            }
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "CacheTypeColumn".to_string()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::N(255))
    }
}

impl sea_orm::sea_query::Nullable for CacheTypeColumn {
    fn null() -> Value {
        Value::String(None)
    }
}