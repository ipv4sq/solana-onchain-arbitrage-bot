use crate::arb::convention::chain::AccountState;
use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::global::enums::step_type::StepType::AccountUpdateDebounced;
use crate::arb::global::trace::types::{Trace, WithTrace};
use crate::arb::pipeline::event_processor::new_pool_processor::NewPoolProcessor;
use crate::arb::pipeline::event_processor::pool_update_processor::PoolUpdateProcessor;
use crate::arb::pipeline::swap_changes::account_monitor::pool_update::PoolUpdate;
use crate::arb::pipeline::swap_changes::account_monitor::trigger::Trigger;
use crate::arb::sdk::yellowstone::GrpcAccountUpdate;
use crate::arb::util::structs::buffered_debouncer::BufferedDebouncer;
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::lazy_arc;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

#[allow(non_upper_case_globals)]
static LastPoolFromGrpc: LazyCache<Pubkey, AccountState> = LazyCache::new();

#[allow(non_upper_case_globals)]
pub static OwnerAccountDebouncer: Lazy<
    Arc<BufferedDebouncer<Pubkey, WithTrace<GrpcAccountUpdate>>>,
> = lazy_arc!(BufferedDebouncer::new(
    Duration::from_millis(1),
    route_pool_update,
));

async fn route_pool_update(update: WithTrace<GrpcAccountUpdate>) {
    let WithTrace(update, trace) = update;
    let updated = AccountState::from_grpc_update(&update);
    let previous = LastPoolFromGrpc.put(update.account, updated.clone());
    let pool_update = PoolUpdate {
        previous,
        current: updated,
    };
    trace.step_with_address(AccountUpdateDebounced, "account_address", update.account);
    if PoolRecordRepository::is_pool_recorded(pool_update.pool()).await {
        if let Err(e) = PoolUpdateProcessor
            .publish(WithTrace(Trigger::PoolUpdate(pool_update), trace))
            .await
        {
            error!("Failed to publish pool update: {}", e);
        }
    } else {
        if let Err(e) = NewPoolProcessor
            .publish(WithTrace(*pool_update.pool(), trace))
            .await
        {
            error!("Failed to publish new pool update: {}", e);
        }
    }
}
