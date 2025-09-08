use crate::convention::chain::mapper::traits::ToUnified;
use crate::global::constant::mev_bot::MevBot;
use crate::pipeline::event_processor::mev_bot_processor::MevBotTxProcessor;
use crate::sdk::yellowstone::{GrpcTransactionUpdate, SolanaGrpcClient, TransactionFilter};
use crate::unit_ok;
use anyhow::Result;
use tracing::info;

pub struct SolanaMevBotOnchainListener {
    client: SolanaGrpcClient,
}

impl SolanaMevBotOnchainListener {
    pub fn new() -> Self {
        Self {
            client: SolanaGrpcClient::from_env().unwrap(),
        }
    }

    pub async fn start(self, auto_retry: bool) -> Result<()> {
        info!(
            "Starting MEV bot subscription for program: {} (auto_retry: {})",
            MevBot::EMV_BOT_PROGRAM.to_string(),
            auto_retry
        );

        let filter = TransactionFilter::new("mev_bot")
            .with_program(&MevBot::EMV_BOT_PROGRAM)
            .include_failed(false)
            .include_votes(false);

        self.client
            .subscribe_transactions(filter, Self::handle_transaction, auto_retry)
            .await
    }

    async fn handle_transaction(tx_update: GrpcTransactionUpdate) -> Result<()> {
        info!("Received transaction: {:?}", tx_update.signature);
        let tx = tx_update.to_unified()?;
        MevBotTxProcessor.try_publish(tx)?;
        unit_ok!()
    }
}

pub async fn start_mev_bot_subscriber() -> Result<()> {
    let subscriber = SolanaMevBotOnchainListener::new();
    subscriber.start(true).await
}
