use crate::convention::chain::util::simulation::SimulationResult;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::enums::step_type::StepType;
use crate::global::trace::types::Trace;
use crate::pipeline::uploader::common::simulation_log;
use crate::pipeline::uploader::provider::jito::send_bundle;
use crate::return_error;
use crate::sdk::rpc::methods::simulation::simulate_transaction_with_config;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;
use tracing::{error, info};

pub async fn simulate_mev_tx(
    tx: &VersionedTransaction,
    trace: &Trace,
) -> anyhow::Result<SimulationResult> {
    if trace.since_begin() > 300 {
        info!(
            "Gave up on simulation tx because it takes {} milliseconds from trigger to now",
            trace.since_begin()
        );
        return_error!("Gave up");
    }
    trace.step(StepType::MevSimulationTxRpcCall);

    // Use the simpler simulate_transaction for better performance
    // Note: This won't return metadata for failed simulations
    let response = simulate_transaction_with_config(
        tx,
        RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: false,
            commitment: Some(CommitmentConfig::processed()),
            encoding: Some(UiTransactionEncoding::Base64),
            accounts: None,
            min_context_slot: Some(trace.slot),
            inner_instructions: false,
        },
    )
    .await?;
    let result = SimulationResult::from(&response.value);
    trace.step(StepType::MevSimulationTxRpcReturned);

    Ok(result)
}

pub async fn real_mev_tx(tx: &VersionedTransaction, trace: &Trace) -> anyhow::Result<String> {
    if trace.since_begin() > 400 {
        info!(
            "Gave up on landing tx because it takes {} milliseconds from trigger to now",
            trace.since_begin()
        );
        return_error!("Gave up");
    }
    trace.step(StepType::MevRealTxRpcCall);
    // let response = rpc_client().send_transaction(tx).await?;
    // sender(tx).await;
    let response = send_bundle(tx).await;
    match response {
        Ok(bundle_id) => {
            trace.step_with(
                StepType::MevRealTxRpcReturned,
                "jito_bundle_id",
                bundle_id.clone(),
            );
            info!("MEV transaction sent successfully: jito id: {}", bundle_id);
            Ok(bundle_id)
        }
        Err(e) => {
            trace.step_with(StepType::MevRealTxRpcReturned, "error", e.to_string());
            error!("Failed to send MEV transaction: {}", e);
            Err(e)
        }
    }
}

pub async fn simulate_and_log_mev(
    owner: Pubkey,
    tx: &VersionedTransaction,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    pools: &[AnyPoolConfig],
    _minimum_profit: u64,
    trace: Trace,
) -> anyhow::Result<(SimulationResult, Trace)> {
    let result = simulate_mev_tx(tx, &trace).await?;

    if let Err(e) = simulation_log::log_mev_simulation(
        &result,
        &trace,
        &owner,
        tx,
        minor_mint,
        desired_mint,
        pools,
    )
    .await
    {
        error!("Failed to log MEV simulation: {}", e);
    }

    Ok((result, trace))
}
