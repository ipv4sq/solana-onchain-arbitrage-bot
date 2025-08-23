use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "arbitrage_results")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub transaction_hash: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: i64,
    pub output_amount: i64,
    pub profit_amount: i64,
    pub profit_percentage: Decimal,
    pub path: Json,  // JSON array of pool IDs
    pub gas_cost: i64,
    pub net_profit: i64,
    pub slot: i64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}