use crate::database::columns::PubkeyTypeString;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mints")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: PubkeyTypeString,
    pub symbol: String,
    pub decimals: i16,
    pub program: PubkeyTypeString,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
