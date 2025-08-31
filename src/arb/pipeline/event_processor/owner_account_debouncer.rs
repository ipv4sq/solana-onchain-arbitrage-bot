use crate::arb::convention::chain::AccountState;
use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::global::enums::step_type::StepType::{AccountUpdateDebounced, DeterminePoolExists};
use crate::arb::global::trace::types::WithTrace;
use crate::arb::pipeline::event_processor::new_pool_processor::NewPoolProcessor;
use crate::arb::pipeline::event_processor::pool_update_processor::PoolUpdateProcessor;
use crate::arb::pipeline::event_processor::structs::pool_update::PoolUpdate;
use crate::arb::pipeline::event_processor::structs::trigger::Trigger;
use crate::arb::sdk::yellowstone::GrpcAccountUpdate;
use crate::arb::util::structs::buffered_debouncer::BufferedDebouncer;
use crate::arb::util::structs::lazy_cache::LazyCache;
use crate::lazy_arc;
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;

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

    let recorded = PoolRecordRepository::is_pool_recorded(pool_update.pool()).await;
    trace.step_with(DeterminePoolExists, "account_address", recorded.to_string());

    if recorded {
        let _ = PoolUpdateProcessor
            .publish(WithTrace(Trigger::PoolUpdate(pool_update), trace))
            .await;
    } else {
        let _ = NewPoolProcessor
            .publish(WithTrace(*pool_update.pool(), trace))
            .await;
    }
}
