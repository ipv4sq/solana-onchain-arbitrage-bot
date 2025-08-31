use crate::arb::convention::chain::AccountState;
use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::global::constant::duration::Interval;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::enums::step_type::StepType::{AccountUpdateDebounced, DeterminePoolExists};
use crate::arb::global::state::account_data_holder::AccountDataHolder;
use crate::arb::global::trace::types::WithTrace;
use crate::arb::pipeline::event_processor::new_pool_processor::NewPoolProcessor;
use crate::arb::pipeline::event_processor::pool_update_processor::PoolUpdateProcessor;
use crate::arb::pipeline::event_processor::structs::pool_update::AccountComparison;
use crate::arb::pipeline::event_processor::structs::trigger::Trigger;
use crate::arb::sdk::yellowstone::GrpcAccountUpdate;
use crate::arb::util::alias::AResult;
use crate::arb::util::structs::buffered_debouncer::BufferedDebouncer;
use crate::arb::util::structs::ttl_loading_cache::TtlLoadingCache;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::lazy_arc;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;

#[allow(non_upper_case_globals)]
pub static OwnerAccountDebouncer: Lazy<
    Arc<BufferedDebouncer<Pubkey, WithTrace<GrpcAccountUpdate>>>,
> = lazy_arc!(BufferedDebouncer::new(
    Duration::from_millis(7),
    send_to_router,
));

async fn send_to_router(update: WithTrace<GrpcAccountUpdate>) {
    AccountUpdateRouteProcessor.publish(update).await.ok();
}

#[allow(non_upper_case_globals)]
pub static AccountUpdateRouteProcessor: Lazy<Arc<PubSubProcessor<WithTrace<GrpcAccountUpdate>>>> =
    lazy_arc!({
        PubSubProcessor::new(
            PubSubConfig {
                worker_pool_size: 16,
                channel_buffer_size: 50000,
                name: "AccountUpdateRouteProcessor".to_string(),
            },
            route_pool_update,
        )
    });

async fn route_pool_update(update: WithTrace<GrpcAccountUpdate>) -> AResult<()> {
    let WithTrace(update, trace) = update;
    let previous = LastAccountUpdateCache.get_sync(&update.account);
    let updated = AccountState::from_grpc_update(&update);
    LastAccountUpdateCache
        .put(update.account, updated.clone())
        .await;

    let comparison = AccountComparison {
        previous,
        current: updated,
    };
    trace.step_with_address(AccountUpdateDebounced, "account_address", update.account);

    let recorded = PoolRecordRepository::is_pool_recorded(comparison.pool()).await;
    trace.step_with(DeterminePoolExists, "account_address", recorded.to_string());

    match update.owner {
        PoolProgram::METEORA_DLMM => {
            // this is for caching bin arrays
            AccountDataHolder::update(comparison.current.pubkey, comparison.current.data.clone())
                .await;
        }
        PoolProgram::METEORA_DAMM_V2 => {}
        PoolProgram::PUMP_AMM => {}
        PoolProgram::RAYDIUM_CPMM => {}
        _ => {}
    }

    if recorded {
        let _ = PoolUpdateProcessor
            .publish(WithTrace(Trigger::AccountCompare(comparison), trace))
            .await;
    } else {
        let _ = NewPoolProcessor
            .publish(WithTrace(*comparison.pool(), trace))
            .await;
    }
    Ok(())
}

#[allow(non_upper_case_globals)]
static LastAccountUpdateCache: Lazy<TtlLoadingCache<Pubkey, AccountState>> =
    Lazy::new(|| TtlLoadingCache::new(10_000_000, Interval::HOUR, |_| async move { None }));
