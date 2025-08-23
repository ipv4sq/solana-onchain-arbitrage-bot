use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use crate::arb::constant::dex_type::DexType;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pool_mints")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pool_id: String,
    pub desired_mint: String,
    pub the_other_mint: String,
    pub dex_type: DexType,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}