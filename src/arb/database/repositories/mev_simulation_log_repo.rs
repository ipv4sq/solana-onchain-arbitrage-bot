use crate::arb::database::columns::PubkeyType;
use crate::arb::database::entity::mev_simulation_log::{
    MevSimulationLogDetails, MevSimulationLogParams, ReturnData, SimulationAccount,
};
use crate::arb::database::entity::{mev_simulation_log, MevSimulationLog, MevSimulationLogTable};
use crate::arb::global::db::get_db;
use anyhow::Result;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub struct MevSimulationLogRepository;

impl MevSimulationLogRepository {
    pub async fn insert_from_account_metas(
        params: MevSimulationLogParams,
        accounts: Vec<AccountMeta>,
    ) -> Result<MevSimulationLog> {
        let mut new_params = params;
        new_params.details.accounts = accounts
            .into_iter()
            .map(|acc| SimulationAccount {
                pubkey: acc.pubkey,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect();

        Self::insert(new_params).await
    }
    pub async fn insert(params: MevSimulationLogParams) -> Result<MevSimulationLog> {
        let model = mev_simulation_log::ActiveModel {
            id: NotSet,
            minor_mint: Set(PubkeyType::from(params.minor_mint)),
            desired_mint: Set(PubkeyType::from(params.desired_mint)),
            minor_mint_sym: Set(params.minor_mint_sym),
            desired_mint_sym: Set(params.desired_mint_sym),
            pools: Set(params.pools),
            profitable: Set(params.profitable),
            details: Set(params.details),
            profitability: Set(params.profitability),
            tx_size: Set(params.tx_size),
            simulation_status: Set(params.simulation_status),
            compute_units_consumed: Set(params.compute_units_consumed),
            error_message: Set(params.error_message),
            logs: Set(params.logs),
            return_data: Set(params.return_data),
            units_per_byte: Set(params.units_per_byte),
            trace: Set(params.trace),
            created_at: NotSet,
            updated_at: NotSet,
        };

        let db = get_db();
        let result = MevSimulationLogTable::insert(model).exec(db).await?;

        Self::find_by_id(result.last_insert_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve inserted record"))
    }

    pub async fn find_by_id(id: i32) -> Result<Option<MevSimulationLog>> {
        let db = get_db();
        Ok(MevSimulationLogTable::find_by_id(id).one(db).await?)
    }

    pub async fn find_by_mints(
        minor_mint: Pubkey,
        desired_mint: Pubkey,
    ) -> Result<Vec<MevSimulationLog>> {
        let db = get_db();
        Ok(MevSimulationLogTable::find()
            .filter(mev_simulation_log::Column::MinorMint.eq(PubkeyType::from(minor_mint)))
            .filter(mev_simulation_log::Column::DesiredMint.eq(PubkeyType::from(desired_mint)))
            .order_by_desc(mev_simulation_log::Column::CreatedAt)
            .all(db)
            .await?)
    }

    pub async fn find_profitable(limit: u64) -> Result<Vec<MevSimulationLog>> {
        let db = get_db();
        let paginator = MevSimulationLogTable::find()
            .filter(mev_simulation_log::Column::Profitable.eq(true))
            .order_by_desc(mev_simulation_log::Column::CreatedAt)
            .paginate(db, limit);
        Ok(paginator.fetch_page(0).await?)
    }

    pub async fn find_recent(limit: u64) -> Result<Vec<MevSimulationLog>> {
        let db = get_db();
        let paginator = MevSimulationLogTable::find()
            .order_by_desc(mev_simulation_log::Column::CreatedAt)
            .paginate(db, limit);
        Ok(paginator.fetch_page(0).await?)
    }

    pub async fn count_total() -> Result<u64> {
        let db = get_db();
        Ok(MevSimulationLogTable::find().count(db).await?)
    }

    pub async fn count_profitable() -> Result<u64> {
        let db = get_db();
        Ok(MevSimulationLogTable::find()
            .filter(mev_simulation_log::Column::Profitable.eq(true))
            .count(db)
            .await?)
    }

    pub async fn find_failed_simulations(limit: u64) -> Result<Vec<MevSimulationLog>> {
        let db = get_db();
        let paginator = MevSimulationLogTable::find()
            .filter(mev_simulation_log::Column::SimulationStatus.eq("failed"))
            .order_by_desc(mev_simulation_log::Column::CreatedAt)
            .paginate(db, limit);
        Ok(paginator.fetch_page(0).await?)
    }
}
