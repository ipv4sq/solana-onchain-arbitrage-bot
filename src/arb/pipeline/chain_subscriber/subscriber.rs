use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::global::constant::mev_bot::MevBot;
use crate::arb::pipeline::event_processor::mev_bot_processor::MevBotTxProcessor;
use crate::arb::sdk::yellowstone::{GrpcTransactionUpdate, SolanaGrpcClient, TransactionFilter};
use anyhow::Result;
use tracing::info;

pub struct SolanaMevBotOnchainListener {
    client: SolanaGrpcClient,
}

impl SolanaMevBotOnchainListener {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
        })
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
            .subscribe_transactions(
                filter,
                |tx_update| async move { Self::handle_transaction(tx_update).await },
                auto_retry,
            )
            .await
    }

    async fn handle_transaction(tx_update: GrpcTransactionUpdate) -> Result<()> {
        info!("Received transaction: {:?}", tx_update.signature);
        if let Err(e) = tx_update
            .to_unified()
            .and_then(|t| MevBotTxProcessor.try_publish(t))
        {
            tracing::error!("Failed to publish SMB transaction: {} to the consumer", e);
        }
        Ok(())
    }
}

pub async fn start_mev_bot_subscriber() -> Result<()> {
    let subscriber = SolanaMevBotOnchainListener::from_env()?;
    subscriber.start(true).await // auto_retry = true
}
