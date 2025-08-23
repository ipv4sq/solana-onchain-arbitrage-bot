use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pool_metrics")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pool_id: String,
    pub dex_type: String,
    pub tvl_usd: Decimal,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub fee_24h_usd: Decimal,
    pub apy_24h: Decimal,
    pub price_impact_2_percent: Decimal,  // Price impact for 2% of TVL trade
    pub swap_count_24h: i64,
    pub unique_traders_24h: i32,
    pub last_swap_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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