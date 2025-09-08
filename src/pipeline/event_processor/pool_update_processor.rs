use crate::global::enums::step_type::StepType;
use crate::global::state::any_pool_holder::AnyPoolHolder;
use crate::global::trace::types::WithTrace;
use crate::pipeline::event_processor::structs::trigger::Trigger;
use crate::pipeline::trade_strategy::entry::on_pool_update;
use crate::util::traits::option::OptionExt;
use crate::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tracing::info;

#[allow(non_upper_case_globals)]
pub static PoolUpdateProcessor: Lazy<Arc<PubSubProcessor<WithTrace<Trigger>>>> = lazy_arc!({
    PubSubProcessor::new(
        PubSubConfig {
            worker_pool_size: 32,
            channel_buffer_size: 50000,
            name: "PoolUpdateProcessor".to_string(),
        },
        process_pool_update,
    )
});

pub async fn process_pool_update(update: WithTrace<Trigger>) -> anyhow::Result<()> {
    let WithTrace(trigger, trace) = update;

    let pool_addr = *trigger.pool();
    trace.step(StepType::ReceivePoolUpdate);

    match trigger {
        Trigger::AccountCompare(update) => {
            // update pool
            let updated_config = AnyPoolHolder::update_config(
                update.pool(),
                &update.current.owner,
                &update.current.data,
            )
            .await?;
            info!("Pool data changed for: {}", pool_addr);
            on_pool_update(pool_addr, updated_config, trace).await;
        }
        Trigger::PoolAddress(addr) => {
            info!("Pool triggered directly for: {}", addr);
            let updated_config = AnyPoolHolder::get(&pool_addr).await.or_err("")?;
            on_pool_update(pool_addr, updated_config, trace).await;
        }
    }

    unit_ok!()
}
