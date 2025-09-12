use crate::global::enums::step_type::StepType;
use crate::global::trace::types::Trace;
use crate::pipeline::uploader::provider::helius::facade::{
    build_helius_jito_tip_ix, build_helius_swqos_tip_ix,
};
use crate::pipeline::uploader::provider::jito::facade::send_bundle;
use crate::return_error;
use crate::util::alias::{AResult, Lamport, Literal};
use jito::facade::build_jito_tip_ix;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::transaction::VersionedTransaction;
use tracing::{error, info};

pub mod helius;
pub mod jito;
pub mod shyft;

pub enum SenderChannel {
    HeliusSwqos,
    Jito,
    HeliusJito,
    Shyft,
}

impl SenderChannel {
    pub fn tip_ix(&self, payer: &Pubkey, unit_price: Lamport) -> (Vec<Instruction>, Literal) {
        let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(unit_price);
        match self {
            SenderChannel::Jito => build_jito_tip_ix(payer),
            SenderChannel::HeliusSwqos => {
                let (mut ix, tip) = build_helius_swqos_tip_ix(payer);
                ix.push(priority_fee_ix);
                (ix, tip)
            }
            SenderChannel::HeliusJito => {
                let (mut ix, tip) = build_helius_jito_tip_ix(payer);
                ix.push(priority_fee_ix);
                (ix, tip)
            }
            SenderChannel::Shyft => (vec![priority_fee_ix.into()], 0f64),
        }
    }

    pub async fn send_tx(&self, tx: &VersionedTransaction, trace: &Trace) -> AResult<()> {
        if trace.since_begin() > 400 {
            info!(
                "Gave up on landing tx because it takes {} milliseconds from trigger to now",
                trace.since_begin()
            );
            return_error!("Gave up");
        }
        trace.step(StepType::MevRealTxRpcCall);
        match self {
            SenderChannel::HeliusSwqos => {}
            SenderChannel::Jito => {
                let response = send_bundle(tx).await;
                let _ = match response {
                    Ok(bundle_id) => {
                        trace.step_with(
                            StepType::MevRealTxRpcReturned,
                            "jito_bundle_id",
                            bundle_id.clone(),
                        );
                        info!("MEV transaction sent successfully: jito id: {}", bundle_id);
                    }
                    Err(e) => {
                        trace.step_with(StepType::MevRealTxRpcReturned, "error", e.to_string());
                        error!("Failed to send MEV transaction: {}", e);
                    }
                };
                todo!()
            }
            SenderChannel::HeliusJito => {}
            SenderChannel::Shyft => {}
        }
        todo!()
    }
}
