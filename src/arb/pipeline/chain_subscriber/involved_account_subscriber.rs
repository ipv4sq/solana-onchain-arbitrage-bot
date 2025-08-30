use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::trace::types::Trace;
use crate::arb::pipeline::event_processor::involved_account_processor::InvolvedAccountTxDebouncer;
use crate::arb::pipeline::swap_changes::account_monitor::subscriber::{
    NEW_POOL_CONSUMER, POOL_UPDATE_CONSUMER,
};
use crate::arb::pipeline::swap_changes::account_monitor::trigger::Trigger;
use crate::arb::sdk::yellowstone::{GrpcTransactionUpdate, SolanaGrpcClient, TransactionFilter};
use crate::arb::util::structs::buffered_debouncer::BufferedDebouncer;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

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
        let mut filter = TransactionFilter::new("involved_accounts");
        filter
            .account_include
            .push(PoolProgram::PUMP_AMM.to_string());

        self.client
            .subscribe_transactions(filter, Self::handle_transaction_update, true)
            .await
    }

    async fn handle_transaction_update(update: GrpcTransactionUpdate) -> Result<()> {
        let trace = Trace::new();

        trace.step_with(
            StepType::Custom("TransactionReceived".to_string()),
            "signature",
            &update.signature,
        );
        InvolvedAccountTxDebouncer.update(update.signature.clone(), (update, trace));
        unit_ok!()
    }
}

pub async fn start_involved_account_monitor() -> Result<()> {
    let subscriber = InvolvedAccountSubscriber::new();
    subscriber.start().await
}
