use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::convention::chain::Transaction;
use crate::arb::global::constant::mev_bot::MevBot;
use crate::arb::pipeline::pool_indexer::mev_bot::consumer::MEV_TX_CONSUMER;
use crate::arb::sdk::yellowstone::{GrpcTransactionUpdate, SolanaGrpcClient, TransactionFilter};
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use tracing::info;

pub struct SolanaMevBotOnchainListener {
    client: SolanaGrpcClient,
    program_id: Pubkey,
}

impl SolanaMevBotOnchainListener {
    pub fn from_env(program_id: Pubkey) -> Result<Self> {
        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
            program_id,
        })
    }

    pub async fn start(self, auto_retry: bool) -> Result<()> {
        info!(
            "Starting MEV bot subscription for program: {} (auto_retry: {})",
            self.program_id, auto_retry
        );

        let filter = TransactionFilter::new("mev_bot")
            .with_program(&self.program_id)
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
        if let Err(e) = tx_update.to_unified().and_then(try_publish_mev_transaction) {
            tracing::error!("Failed to publish SMB transaction: {} to the consumer", e);
        }
        Ok(())
    }
}

pub async fn start_mev_bot_subscriber() -> Result<()> {
    let subscriber = SolanaMevBotOnchainListener::from_env(MevBot::EMV_BOT_PROGRAM)?;
    subscriber.start(true).await // auto_retry = true
}

pub async fn publish_mev_transaction(tx: Transaction) -> Result<()> {
    MEV_TX_CONSUMER.publish(tx).await
}

pub fn try_publish_mev_transaction(tx: Transaction) -> Result<()> {
    MEV_TX_CONSUMER.try_publish(tx)
}
