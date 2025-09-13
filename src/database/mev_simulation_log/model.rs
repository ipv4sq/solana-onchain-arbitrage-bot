use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mev_simulation_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub minor_mint_sym: String,
    pub desired_mint_sym: String,
    pub pools: Vec<String>,
    pub pool_types: Vec<String>,
    #[sea_orm(column_type = "Json")]
    pub details: MevSimulationLogDetails,
    pub tx_size: Option<i32>,
    pub simulation_status: Option<String>,
    pub compute_units_consumed: Option<i64>,
    pub error_message: Option<String>,
    pub logs: Option<Vec<String>>,
    pub trace: Option<JsonValue>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct MevSimulationLogDetails {
    pub accounts: Vec<SimulationAccount>,
    pub minor_mint: String,
    pub desired_mint: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationAccount {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MevSimulationLogParams {
    pub minor_mint: String,
    pub desired_mint: String,
    pub minor_mint_sym: String,
    pub desired_mint_sym: String,
    pub pools: Vec<String>,
    pub pool_types: Vec<String>,
    pub details: MevSimulationLogDetails,
    pub tx_size: Option<i32>,
    pub simulation_status: Option<String>,
    pub compute_units_consumed: Option<i64>,
    pub error_message: Option<String>,
    pub logs: Option<Vec<String>>,
    pub trace: Option<JsonValue>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}