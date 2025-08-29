use crate::arb::database::columns::CacheTypeColumn;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "kv_cache")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_name = "type")]
    pub r#type: CacheTypeColumn,
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: JsonValue,
    pub valid_until: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
