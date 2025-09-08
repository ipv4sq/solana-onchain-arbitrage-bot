use crate::convention::chain::mapper::traits::ToUnified;
use crate::database::pool_record::repository::PoolRecordRepository;
use crate::dex::interface::PoolConfig;
use crate::dex::pump_amm::config::PumpAmmConfig;
use crate::global::enums::step_type::StepType;
use crate::global::trace::types::{Trace, WithTrace};
use crate::pipeline::event_processor::new_pool_processor::NewPoolProcessor;
use crate::pipeline::event_processor::pool_update_processor::PoolUpdateProcessor;
use crate::pipeline::event_processor::structs::trigger::Trigger;
use crate::pipeline::event_processor::token_balance::token_balance_processor::process_token_balance_change;
use crate::sdk::yellowstone::GrpcTransactionUpdate;
use crate::util::alias::{AResult, PoolAddress};
use crate::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Arc;

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

pub async fn process_involved_account_transaction(update: TxWithTrace) -> AResult<()> {
    let (update, trace) = update;
    trace.step_with(
        StepType::Custom("ProcessingTransaction".to_string()),
        "signature",
        &update.signature,
    );

    let tx = update.to_unified()?;
    let ixs = tx.all_instructions();
    // here I am going to cache the balance changes:
    // this is for raydium vault
    // and for pump amm vault
    let _ = process_token_balance_change(tx, &trace).await;

    let pump_pools: HashSet<PoolAddress> = ixs
        .iter()
        .flat_map(|ix| PumpAmmConfig::pase_swap_from_ix(ix))
        .map(|swap| swap.1)
        .collect();

    if pump_pools.len() > ixs.len() * 2 {
        panic!("There must be something wrong here")
    }

    for pool in pump_pools {
        match PoolRecordRepository::is_pool_recorded(&pool).await {
            true => {
                PoolUpdateProcessor
                    .publish(WithTrace(Trigger::PoolAddress(pool), trace.clone()))
                    .await?
            }
            false => {
                NewPoolProcessor
                    .publish(WithTrace(Trigger::PoolAddress(pool), trace.clone()))
                    .await?
            }
        }
    }

    unit_ok!()
}

#[cfg(test)]
mod tests {
    use crate::sdk::solana_rpc::utils::fetch_tx;

    #[tokio::test]
    async fn test_find_pump_swap_pool() {
        let tx = fetch_tx("4h61Rg4QEGEjCV2T5dEKa3JitKeHGnRz1voH3ypnvtVQHnmZPprtNZemU2G9EwB2TPUSuJL3sFyMK5y5V5QnGH2A").await.unwrap();
        let _inners = tx.just_inner().map_or(vec![], |inner| {
            inner.iter().flat_map(|x| &x.instructions).collect()
        });
    }
}
