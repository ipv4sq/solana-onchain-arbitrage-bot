use crate::convention::chain::util::simulation::SimulationResult;
use crate::database::mev_simulation_log::repository::MevSimulationLogRepository;
use crate::database::mev_simulation_log::{
    MevSimulationLogDetails, MevSimulationLogParams, SimulationAccount,
};
use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::trace::types::Trace;
use crate::util::alias::AResult;
use solana_program::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use tracing::error;

pub async fn log_mev_simulation(
    result: &SimulationResult,
    trace: &Trace,
    owner: &Pubkey,
    tx: &VersionedTransaction,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    pools: &[AnyPoolConfig],
) -> AResult<()> {
    let tx_size = bincode::serialize(tx)?.len() as i32;
    let simulation_status = get_simulation_status(result);

    let accounts = extract_transaction_accounts(tx);

    let (minor_mint_sym, desired_mint_sym) = fetch_mint_symbols(minor_mint, desired_mint).await?;

    let params = build_simulation_log_params(
        result,
        trace,
        minor_mint,
        desired_mint,
        minor_mint_sym,
        desired_mint_sym,
        pools,
        accounts,
        tx_size,
        simulation_status,
    );

    if let Err(e) = MevSimulationLogRepository::insert(params).await {
        error!("Failed to log MEV simulation: {}", e);
    }

    Ok(())
}

fn get_simulation_status(result: &SimulationResult) -> &'static str {
    if result.err.is_some() {
        "failed"
    } else {
        "success"
    }
}

fn extract_transaction_accounts(tx: &VersionedTransaction) -> Vec<SimulationAccount> {
    let (account_keys, header) = match &tx.message {
        solana_sdk::message::VersionedMessage::Legacy(msg) => (&msg.account_keys, &msg.header),
        solana_sdk::message::VersionedMessage::V0(msg) => (&msg.account_keys, &msg.header),
    };

    account_keys
        .iter()
        .enumerate()
        .map(|(idx, pubkey)| {
            let is_signer = idx < header.num_required_signatures as usize;
            let is_writable = if is_signer {
                idx < (header.num_required_signatures - header.num_readonly_signed_accounts)
                    as usize
            } else {
                let non_signer_idx = idx - header.num_required_signatures as usize;
                let num_writable_unsigned = account_keys.len()
                    - header.num_required_signatures as usize
                    - header.num_readonly_unsigned_accounts as usize;
                non_signer_idx < num_writable_unsigned
            };
            SimulationAccount {
                pubkey: pubkey.to_string(),
                is_signer,
                is_writable,
            }
        })
        .collect()
}

async fn fetch_mint_symbols(
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
) -> anyhow::Result<(String, String)> {
    let minor_mint_record = MintRecordRepository::get_mint_or_err(minor_mint).await?;
    let desired_mint_record = MintRecordRepository::get_mint_or_err(desired_mint).await?;
    Ok((minor_mint_record.repr, desired_mint_record.repr))
}

fn build_simulation_log_params(
    result: &SimulationResult,
    trace: &Trace,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    minor_mint_sym: String,
    desired_mint_sym: String,
    pools: &[AnyPoolConfig],
    accounts: Vec<SimulationAccount>,
    tx_size: i32,
    simulation_status: &str,
) -> MevSimulationLogParams {
    MevSimulationLogParams {
        minor_mint: minor_mint.to_string(),
        desired_mint: desired_mint.to_string(),
        minor_mint_sym,
        desired_mint_sym,
        pools: pools.iter().map(|p| p.pool_address().to_string()).collect(),
        pool_types: pools.iter().map(|p| p.dex_type().to_string()).collect(),
        details: MevSimulationLogDetails {
            accounts,
            minor_mint: minor_mint.to_string(),
            desired_mint: desired_mint.to_string(),
        },
        tx_size: Some(tx_size),
        simulation_status: Some(simulation_status.to_string()),
        compute_units_consumed: result.units_consumed.map(|u| u as i64),
        error_message: result.err.clone(),
        logs: Some(result.logs.clone()),
        trace: Some(trace.dump_json()),
    }
}
