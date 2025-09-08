use crate::database::pool_record::model::PoolRecordDescriptor;
use sea_orm::sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr};
use sea_orm::{DbErr, QueryResult, TryGetError, TryGetable, Value};

impl From<PoolRecordDescriptor> for Value {
    fn from(desc: PoolRecordDescriptor) -> Self {
        Value::Json(Some(Box::new(serde_json::to_value(desc).unwrap())))
    }
}

impl TryGetable for PoolRecordDescriptor {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let json_value = serde_json::Value::try_get(res, pre, col)?;
        serde_json::from_value(json_value)
            .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
    }

    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let json_value = serde_json::Value::try_get_by(res, index)?;
        serde_json::from_value(json_value)
            .map_err(|e| TryGetError::DbErr(DbErr::Type(e.to_string())))
    }
}

impl ValueType for PoolRecordDescriptor {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Json(Some(json)) => serde_json::from_value(*json).map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "PoolRecordDescriptor".to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::Json
    }

    fn column_type() -> ColumnType {
        ColumnType::JsonBinary
    }
}

impl Nullable for PoolRecordDescriptor {
    fn null() -> Value {
        Value::Json(None)
    }
}

impl sea_orm::TryFromU64 for PoolRecordDescriptor {
    fn try_from_u64(_n: u64) -> Result<Self, DbErr> {
        Err(DbErr::Type(
            "PoolRecordDescriptor cannot be created from u64".to_string(),
        ))
    }
}
