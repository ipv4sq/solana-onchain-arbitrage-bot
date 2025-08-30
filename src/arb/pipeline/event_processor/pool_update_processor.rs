use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::trace::types::WithTrace;
use crate::arb::pipeline::swap_changes::account_monitor::trigger::Trigger;
use crate::arb::pipeline::trade_strategy::entry::on_pool_update;
use crate::arb::util::alias::{MintAddress, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tracing::info;

#[allow(non_upper_case_globals)]
pub static PoolUpdateProcessor: Lazy<Arc<PubSubProcessor<WithTrace<Trigger>>>> = lazy_arc!({
    let config = PubSubConfig {
        worker_pool_size: 24,
        channel_buffer_size: 5000,
        name: "PoolUpdateProcessor".to_string(),
    };

    PubSubProcessor::new(config, process_pool_update)
});

pub async fn process_pool_update(update: WithTrace<Trigger>) -> anyhow::Result<()> {
    let WithTrace(trigger, trace) = update;

    let pool_addr = *trigger.pool();
    trace.step(StepType::ReceivePoolUpdate);

    match trigger {
        Trigger::PoolUpdate(update) => {
            if update.is_initial() {
                return Ok(());
            }

            if !update.data_changed() {
                return Ok(());
            }

            info!("Pool data changed for: {}", pool_addr);
            // update pool
            let updated_config = AnyPoolConfig::from_owner_and_data(
                update.pool(),
                &update.current.owner,
                &update.current.data,
            )?;
            on_pool_update(pool_addr, updated_config, trace).await;
        }
        Trigger::PoolAddress(addr) => {
            info!("Pool triggered directly for: {}", addr);
            let updated_config = AnyPoolConfig::from(&pool_addr).await?;
            on_pool_update(pool_addr, updated_config, trace).await;
        }
    }

    unit_ok!()
}

pub async fn get_minor_mint_for_pool(pool: &PoolAddress) -> Option<MintAddress> {
    let pool = PoolRecordRepository::get_pool_by_address(pool).await?;
    MintPair(pool.base_mint.into(), pool.quote_mint.into())
        .minor_mint()
        .ok()
}
