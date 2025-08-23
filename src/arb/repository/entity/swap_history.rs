use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "swap_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub transaction_hash: String,
    pub pool_id: String,
    pub dex_type: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount_in: i64,
    pub amount_out: i64,
    pub price: f64,
    pub slot: i64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pool_mints::Entity",
        from = "Column::PoolId",
        to = "super::pool_mints::Column::PoolId"
    )]
    PoolMints,
}

impl Related<super::pool_mints::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PoolMints.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}