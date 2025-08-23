use crate::arb::global::enums::dex_type::DexType;
use crate::arb::repository::types::PubkeyType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pools")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub address: PubkeyType,
    pub name: String,
    pub dex_type: DexType,
    pub base_mint: PubkeyType,
    pub quote_mint: PubkeyType,
    pub base_vault: PubkeyType,
    pub quote_vault: PubkeyType,
    #[sea_orm(column_type = "JsonBinary")]
    pub description: PoolRecordDescriptor,
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
