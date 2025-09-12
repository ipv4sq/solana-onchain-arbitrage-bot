use crate::global::enums::step_type::StepType;
use crate::global::trace::types::Trace;
use crate::pipeline::uploader::provider::jito::send_bundle;
use crate::return_error;
use solana_sdk::transaction::VersionedTransaction;
use tracing::{error, info};

pub async fn send_real_mev_tx(tx: &VersionedTransaction, trace: &Trace) -> anyhow::Result<String> {
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
