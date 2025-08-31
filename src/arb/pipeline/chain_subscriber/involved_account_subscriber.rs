use crate::arb::dex::raydium_cpmm::RAYDIUM_CPMM_AUTHORITY;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::trace::types::Trace;
use crate::arb::pipeline::event_processor::involved_account_processor::InvolvedAccountTxProcessor;
use crate::arb::sdk::yellowstone::{GrpcTransactionUpdate, SolanaGrpcClient, TransactionFilter};
use crate::unit_ok;
use anyhow::Result;
use tracing::info;

pub struct InvolvedAccountSubscriber {
    client: SolanaGrpcClient,
}

impl InvolvedAccountSubscriber {
    pub fn new() -> Self {
        Self {
            client: SolanaGrpcClient::from_env().unwrap(),
        }
    }

    pub async fn start(self) -> Result<()> {
        info!(
            "Starting transaction subscription for {} involved accounts",
            PoolProgram::PUMP_AMM,
        );

        self.client
            .subscribe_transactions(
                TransactionFilter::new("involved_accounts")
                    .with_programs(&vec![PoolProgram::PUMP_AMM, RAYDIUM_CPMM_AUTHORITY]),
                Self::handle_transaction_update,
                true,
            )
            .await
    }

    async fn handle_transaction_update(update: GrpcTransactionUpdate) -> Result<()> {
        let trace = Trace::new();

        trace.step_with(
            StepType::Custom("TransactionReceived".to_string()),
            "signature",
            &update.signature,
        );
        InvolvedAccountTxProcessor.publish((update, trace)).await?;
        unit_ok!()
    }
}

pub async fn start_involved_account_monitor() -> Result<()> {
    let subscriber = InvolvedAccountSubscriber::new();
    subscriber.start().await
}
