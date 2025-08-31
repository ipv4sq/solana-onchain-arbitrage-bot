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
use crate::arb::pipeline::event_processor::token_balance::token_balance_processor::TokenBalanceProcessor;
use crate::arb::sdk::yellowstone::GrpcTransactionUpdate;
use crate::arb::util::alias::{AResult, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use once_cell::sync::Lazy;
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

pub async fn process_involved_account_transaction(update: TxWithTrace) -> AResult<()> {
    let (update, trace) = update;
    trace.step_with(
        StepType::Custom("ProcessingTransaction".to_string()),
        "signature",
        &update.signature,
    );
    info!("Processing involved account transaction");

    let tx = update.to_unified()?;
    let ixs = tx.all_instructions();

    let pump_pools: Vec<PoolAddress> = ixs
        .iter()
        .filter(|ix| ix.program_id == PoolProgram::PUMP_AMM)
        .flat_map(|x| find_pump_swap(x))
        .collect();

    if !pump_pools.is_empty() {
        info!("Found {} pump pools in transaction", pump_pools.len());
    }

    TokenBalanceProcessor
        .publish(WithTrace(tx, trace.clone()))
        .await?;

    if pump_pools.len() > ixs.len() * 2 {
        panic!("There must be something wrong here")
    }

    // Deduplicate pools
    use std::collections::HashSet;
    let unique_pools: HashSet<PoolAddress> = pump_pools.into_iter().collect();

    for pool in unique_pools {
        match PoolRecordRepository::is_pool_recorded(&pool).await {
            true => {
                PoolUpdateProcessor
                    .publish(WithTrace(Trigger::PoolAddress(pool), trace.clone()))
                    .await?
            }
            false => {
                NewPoolProcessor
                    .publish(WithTrace(pool, trace.clone()))
                    .await?
            }
        }
    }

    unit_ok!()
}

pub fn find_pump_swap(ix: &Instruction) -> Option<PoolAddress> {
    /*
    #1 - Pool:Pump.fun AMM ( USDC-WSOL) Market
    #2 - User:
    #3 - Global Config:
    #4 - Base Mint:
    #5 - Quote Mint:
    */
    if ix.accounts.len() < 6 {
        return None;
    }
    let account_1 = ix.accounts.get(0)?.pubkey;
    let account_3 = ix.accounts.get(2)?.pubkey;
    let account_4 = ix.accounts.get(3)?.pubkey;
    let account_5 = ix.accounts.get(4)?.pubkey;

    if account_3 != PUMP_GLOBAL_CONFIG {
        return None;
    }

    let pair = MintPair(account_4, account_5);
    if !pair.contains(&Mints::WSOL) || !pair.contains(&Mints::USDC) {
        return None;
    }

    Some(account_1)
}

#[cfg(test)]
mod tests {
    use crate::arb::global::state::rpc::fetch_tx;

    #[tokio::test]
    async fn test_find_pump_swap_pool() {
        let tx = fetch_tx("4h61Rg4QEGEjCV2T5dEKa3JitKeHGnRz1voH3ypnvtVQHnmZPprtNZemU2G9EwB2TPUSuJL3sFyMK5y5V5QnGH2A").await.unwrap();
        let _inners = tx.just_inner().map_or(vec![], |inner| {
            inner.iter().flat_map(|x| &x.instructions).collect()
        });
    }
}
