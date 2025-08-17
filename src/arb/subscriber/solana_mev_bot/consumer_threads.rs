use crate::arb::program::solana_mev_bot::subscriber::entry as process_mev_tx;
use crate::arb::subscriber::pubsub::{PubSubConfig, PubSubProcessor};
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;
use std::ops::Deref;
use std::sync::Arc;
use tracing::info;
use crate::arb::chain::Transaction;

const NAME: &str = "SolanaMevBotTransactionDetector";

#[derive(Debug, Clone)]
pub struct MevTransaction {
    pub signature: String,
    pub slot: u64,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub pools: Vec<Pubkey>,
    pub instruction_index: usize,
}

struct MevProcessor(PubSubProcessor<Transaction>);

impl MevProcessor {
    fn new() -> Self {
        let config = PubSubConfig {
            worker_pool_size: 8,
            channel_buffer_size: 1000,
            name: NAME.to_string(),
        };

        let processor = PubSubProcessor::new(config, |tx| {
            Box::pin(async move {
                process_mev_tx(&tx).await?;
                Ok(())
            })
        });

        info!("MEV transaction processor auto-initialized");

        Self(processor)
    }
}

impl Deref for MevProcessor {
    type Target = PubSubProcessor<Transaction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static MEV_PROCESSOR: Lazy<Arc<MevProcessor>> = Lazy::new(|| Arc::new(MevProcessor::new()));

pub async fn publish_mev_transaction(tx: Transaction) -> Result<()> {
    MEV_PROCESSOR.publish(tx).await
}

pub fn try_publish_mev_transaction(tx: Transaction) -> Result<()> {
    MEV_PROCESSOR.try_publish(tx)
}


#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use crate::arb::chain::{Message, TransactionMeta};

    fn create_mock_transaction(slot: u64) -> Transaction {
        Transaction {
            signature: format!("mock_signature_{}", slot),
            slot,
            message: Message {
                account_keys: vec![Pubkey::default()],
                recent_blockhash: "mock_blockhash".to_string(),
                instructions: vec![],
            },
            meta: Some(TransactionMeta {
                fee: 5000,
                compute_units_consumed: Some(100000),
                log_messages: vec![],
                inner_instructions: vec![],
                pre_balances: vec![],
                post_balances: vec![],
                err: None,
            }),
        }
    }

    #[tokio::test]
    async fn test_mev_transaction_processor() {
        for i in 0..10 {
            let tx = create_mock_transaction(12345 + i as u64);
            publish_mev_transaction(tx).await.unwrap();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn test_mev_try_publish() {
        let tx = create_mock_transaction(99999);
        try_publish_mev_transaction(tx).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
