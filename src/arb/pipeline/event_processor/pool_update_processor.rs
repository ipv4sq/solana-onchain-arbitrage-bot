use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::state::any_pool_holder::AnyPoolHolder;
use crate::arb::global::trace::types::WithTrace;
use crate::arb::pipeline::event_processor::structs::trigger::Trigger;
use crate::arb::pipeline::trade_strategy::entry::on_pool_update;
use crate::arb::util::alias::{MintAddress, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;
use crate::arb::util::traits::option::OptionExt;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
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
        Trigger::PoolUpdate(update) => {
            info!("Pool data changed for: {}", pool_addr);
            // update pool
            let updated_config = AnyPoolHolder::update_config(
                update.pool(),
                &update.current.owner,
                &update.current.data,
            )
            .await?;
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
