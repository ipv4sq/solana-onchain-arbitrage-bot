use crate::arb::chain::mapper::traits::ToUnified;
use crate::arb::subscriber::yellowstone::{SolanaGrpcClient, TransactionFilter, TransactionUpdate};
use crate::constants::helpers::ToPubkey;
use crate::constants::mev_bot::SMB_ONCHAIN_PROGRAM_ID;
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use tracing::info;

pub struct MevBotSubscriber {
    client: SolanaGrpcClient,
    program_id: Pubkey,
}

impl MevBotSubscriber {
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

    async fn handle_transaction(tx_update: TransactionUpdate) -> Result<()> {
        use crate::arb::subscriber::solana_mev_bot::consumer::try_publish_mev_transaction as try_publish;
        if let Err(e) = tx_update.to_unified().and_then(try_publish) {
            tracing::error!("Failed to publish SMB transaction: {} to the consumer", e);
        }
        Ok(())
    }
}

pub async fn start_mev_bot_subscriber() -> Result<()> {
    let subscriber = MevBotSubscriber::from_env(SMB_ONCHAIN_PROGRAM_ID.to_pubkey())?;
    subscriber.start(true).await // auto_retry = true
}
