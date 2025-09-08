use crate::global::constant::pool_program::PoolProgram;
use crate::global::enums::step_type::StepType;
use crate::global::trace::types::{Trace, WithTrace};
use crate::pipeline::event_processor::owner_account_debouncer::OwnerAccountDebouncer;
use crate::sdk::yellowstone::{AccountFilter, GrpcAccountUpdate, SolanaGrpcClient};
use crate::unit_ok;
use anyhow::Result;
use tracing::{debug, info};

#[allow(unused)]
pub struct OwnerSubscriber {
    client: SolanaGrpcClient,
}

impl OwnerSubscriber {
    pub fn new() -> Self {
        Self {
            client: SolanaGrpcClient::from_env().unwrap(),
        }
    }

    pub async fn start(self) -> Result<()> {
        info!("Starting owner account subscriber");

        let filter = AccountFilter::new("meteora_pools").with_owners(&[
            PoolProgram::METEORA_DLMM,
            PoolProgram::METEORA_DAMM_V2,
            PoolProgram::PUMP_AMM,
            PoolProgram::RAYDIUM_CPMM,
        ]);

        self.client
            .subscribe_accounts(filter, Self::handle_account_update, true)
            .await
    }

    async fn handle_account_update(update: GrpcAccountUpdate) -> Result<()> {
        let trace = Trace::new(update.slot);
        trace.step_with_address(
            StepType::AccountUpdateReceived,
            "account_address",
            update.account,
        );
        debug!("Pool account update received: {}", update.account);
        OwnerAccountDebouncer.update(update.account, WithTrace(update, trace));

        unit_ok!()
    }
}

pub async fn start_owner_account_monitor() -> Result<()> {
    let monitor = OwnerSubscriber::new();
    monitor.start().await
}
