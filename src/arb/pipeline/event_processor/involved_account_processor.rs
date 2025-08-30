use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::dex::pump_amm::PUMP_GLOBAL_CONFIG;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::trace::types::{Trace, WithTrace};
use crate::arb::pipeline::event_processor::new_pool_processor::NewPoolProcessor;
use crate::arb::pipeline::event_processor::pool_update_processor::PoolUpdateProcessor;
use crate::arb::pipeline::event_processor::structs::trigger::Trigger;
use crate::arb::sdk::yellowstone::GrpcTransactionUpdate;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use anyhow::anyhow;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use tracing::info;

pub type TxWithTrace = (GrpcTransactionUpdate, Trace);

#[allow(non_upper_case_globals)]
pub static InvolvedAccountTxProcessor: Lazy<Arc<PubSubProcessor<TxWithTrace>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 16,
        channel_buffer_size: 10_000,
        name: "InvolvedAccountTransactionProcessor".to_string(),
    };

    PubSubProcessor::new(config, process_involved_account_transaction)
});

pub async fn process_involved_account_transaction(update: TxWithTrace) -> anyhow::Result<()> {
    let (update, trace) = update;
    trace.step_with(
        StepType::Custom("ProcessingTransaction".to_string()),
        "signature",
        &update.signature,
    );
    info!("processing involved account transaction");
    // first, we got to figure out if it's a swap ix.
    let transaction = update.to_unified()?;
    let Some((_ix, inners)) =
        transaction.extract_ix_and_inners(|program_id| *program_id == PoolProgram::PUMP_AMM)
    else {
        return Err(anyhow!("Transaction does not contain involved accounts"));
    };

    if let Some(pool_account) = find_pump_swap_pool(&inners.instructions) {
        info!(
            "Found Pump AMM swap instruction with pool: {}",
            pool_account
        );
        if !PoolRecordRepository::is_pool_recorded(&pool_account).await {
            // this could be a new pool, we did not know, then we try to record it
            NewPoolProcessor
                .publish(WithTrace(pool_account, trace))
                .await?;
        } else {
            // this is something we know, so trigger arbitrage opportunity.
            PoolUpdateProcessor
                .publish(WithTrace(Trigger::PoolAddress(pool_account), trace))
                .await
                .ok();
        }
    }

    unit_ok!()
}

fn find_pump_swap_pool(instructions: &[Instruction]) -> Option<Pubkey> {
    let wsol = Mints::WSOL;

    instructions.iter().find_map(|ix| {
        if ix.accounts.len() < 5 {
            return None;
        }

        if ix.accounts.get(2).map(|acc| acc.pubkey) != Some(PUMP_GLOBAL_CONFIG) {
            return None;
        }

        let has_wsol_at_4_or_5 = ix.accounts.get(3).map(|acc| acc.pubkey) == Some(wsol)
            || ix.accounts.get(4).map(|acc| acc.pubkey) == Some(wsol);

        if has_wsol_at_4_or_5 {
            ix.accounts.get(1).map(|acc| acc.pubkey)
        } else {
            None
        }
    })
}
