use crate::arb::program::solana_mev_bot::subscriber::{entry as process_mev_tx, entry};
use crate::arb::subscriber::pubsub::{PubSubConfig, PubSubProcessor};
use anyhow::Result;
use once_cell::sync::Lazy;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use std::ops::Deref;
use std::sync::Arc;
use tracing::info;

const NAME: &str = "SolanaMevBotTransactionDetector";

struct MevProcessor(PubSubProcessor<EncodedConfirmedTransactionWithStatusMeta>);

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
    type Target = PubSubProcessor<EncodedConfirmedTransactionWithStatusMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static MEV_PROCESSOR: Lazy<Arc<MevProcessor>> = Lazy::new(|| Arc::new(MevProcessor::new()));

pub async fn publish_mev_transaction(tx: EncodedConfirmedTransactionWithStatusMeta) -> Result<()> {
    MEV_PROCESSOR.publish(tx).await
}

pub fn try_publish_mev_transaction(tx: EncodedConfirmedTransactionWithStatusMeta) -> Result<()> {
    MEV_PROCESSOR.try_publish(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::message::MessageHeader;
    use solana_transaction_status::{
        EncodedTransaction, EncodedTransactionWithStatusMeta, UiMessage, UiRawMessage,
        UiTransaction,
    };

    #[tokio::test]
    async fn test_mev_transaction_processor() {
        for i in 0..10 {
            let mock_tx = EncodedConfirmedTransactionWithStatusMeta {
                slot: 12345 + i as u64,
                transaction: EncodedTransactionWithStatusMeta {
                    transaction: EncodedTransaction::Json(UiTransaction {
                        signatures: vec![],
                        message: UiMessage::Raw(UiRawMessage {
                            header: MessageHeader {
                                num_required_signatures: 1,
                                num_readonly_signed_accounts: 0,
                                num_readonly_unsigned_accounts: 0,
                            },
                            account_keys: vec![],
                            recent_blockhash: String::new(),
                            instructions: vec![],
                            address_table_lookups: None,
                        }),
                    }),
                    meta: None,
                    version: None,
                },
                block_time: None,
            };

            publish_mev_transaction(mock_tx).await.unwrap();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn test_mev_try_publish() {
        let mock_tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 99999,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec![],
                    message: UiMessage::Raw(UiRawMessage {
                        header: MessageHeader {
                            num_required_signatures: 1,
                            num_readonly_signed_accounts: 0,
                            num_readonly_unsigned_accounts: 0,
                        },
                        account_keys: vec![],
                        recent_blockhash: String::new(),
                        instructions: vec![],
                        address_table_lookups: None,
                    }),
                }),
                meta: None,
                version: None,
            },
            block_time: None,
        };

        try_publish_mev_transaction(mock_tx).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
