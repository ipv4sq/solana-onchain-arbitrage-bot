use crate::arb::constant::dex_type::DexType;
use crate::arb::repository::types::pubkey_type::PubkeyWrapper;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pools")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: PubkeyWrapper,
    pub name: String,
    pub dex_type: DexType,
    pub base_mint: PubkeyWrapper,
    pub quote_mint: PubkeyWrapper,
    pub base_vault: PubkeyWrapper,
    pub quote_vault: PubkeyWrapper,
    #[sea_orm(column_type = "JsonBinary")]
    pub description: Json,
    #[sea_orm(column_type = "JsonBinary")]
    pub data_snapshot: Json,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PoolRecordDescriptor {
    pub base_symbol: String,
    pub quote_symbol: String,
    pub base: Pubkey,
    pub quote: Pubkey,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
