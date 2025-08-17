use crate::arb::chain::mapper::traits::ToUnified;
use crate::arb::subscriber::yellowstone::{SolanaGrpcClient, TransactionFilter, GrpcTransactionUpdate};
use crate::constants::helpers::ToPubkey;
use crate::constants::mev_bot::EMV_BOT_PROGRAM_ID;
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
        use crate::arb::program::mev_bot::onchain_monitor::consumer::try_publish_mev_transaction as try_publish;
        info!("Received transaction: {:?}", tx_update.signature);
        if let Err(e) = tx_update.to_unified().and_then(try_publish) {
            tracing::error!("Failed to publish SMB transaction: {} to the consumer", e);
        }
        Ok(())
    }
}

pub async fn start_mev_bot_subscriber() -> Result<()> {
    let subscriber = SolanaMevBotOnchainListener::from_env(EMV_BOT_PROGRAM_ID.to_pubkey())?;
    subscriber.start(true).await // auto_retry = true
}
