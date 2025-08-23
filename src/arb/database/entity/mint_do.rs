use crate::arb::database::columns::PubkeyType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mints")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: PubkeyType,
    pub symbol: String,
    pub decimals: i16,
    pub program: PubkeyType,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
