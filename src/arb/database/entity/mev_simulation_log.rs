use crate::arb::database::columns::PubkeyType;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mev_simulation_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub minor_mint: PubkeyType,
    pub desired_mint: PubkeyType,
    pub minor_mint_sym: String,
    pub desired_mint_sym: String,
    pub pools: Vec<String>,
    pub profitable: bool,
    pub details: MevSimulationLogDetails,
    pub profitability: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct MevSimulationLogDetails {
    pub accounts: Vec<SimulationAccount>,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub simulation_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MevSimulationLogParams {
    pub minor_mint: Pubkey,
    pub desired_mint: Pubkey,
    pub minor_mint_sym: String,
    pub desired_mint_sym: String,
    pub pools: Vec<String>,
    pub profitable: bool,
    pub profitability: Option<i64>,
    pub details: MevSimulationLogDetails,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
