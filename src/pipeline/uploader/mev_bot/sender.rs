use crate::convention::chain::util::simulation::SimulationResult;
use crate::global::enums::step_type::StepType;
use crate::global::trace::types::Trace;
use crate::pipeline::uploader::common::simulation_log;
use crate::return_error;
use crate::sdk::rpc::methods::simulation::simulate_transaction_with_config;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::UiTransactionEncoding;
use tracing::info;

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
