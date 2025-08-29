use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::convention::chain::mapper::traits::ToUnified;
use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::constant::pool_program::PoolProgram;
use crate::arb::global::trace::types::{StepType, Trace};
use crate::arb::pipeline::swap_changes::account_monitor::subscriber::NEW_POOL_CONSUMER;
use crate::arb::sdk::yellowstone::{GrpcTransactionUpdate, SolanaGrpcClient, TransactionFilter};
use crate::arb::util::structs::buffered_debouncer::BufferedDebouncer;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::arb::util::worker::pubsub::{PubSubConfig, PubSubProcessor};
use crate::{lazy_arc, unit_ok};
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

static TRANSACTION_PROCESSOR: Lazy<Arc<PubSubProcessor<(GrpcTransactionUpdate, Trace)>>> =
    lazy_arc!({
        let config = PubSubConfig {
            worker_pool_size: 16,
            channel_buffer_size: 10_000,
            name: "InvolvedAccountTransactionProcessor".to_string(),
        };

        PubSubProcessor::new(config, |(update, trace): (GrpcTransactionUpdate, Trace)| {
            Box::pin(async move {
                process_involved_account_transaction(update, trace).await?;
                Ok(())
            })
        })
    });

static TRANSACTION_DEBOUNCER: Lazy<Arc<BufferedDebouncer<String, (GrpcTransactionUpdate, Trace)>>> =
    lazy_arc!({
        BufferedDebouncer::new(
            Duration::from_millis(1),
            |(update, trace): (GrpcTransactionUpdate, Trace)| async move {
                trace.step_with(
                    StepType::Custom("TransactionDebounced".to_string()),
                    "signature",
                    &update.signature,
                );
                if let Err(e) = TRANSACTION_PROCESSOR.publish((update, trace)).await {
                    error!("Failed to publish transaction update: {}", e);
                }
            },
        )
    });

pub struct InvolvedAccountSubscriber {
    client: SolanaGrpcClient,
    target_accounts: Vec<Pubkey>,
}

impl InvolvedAccountSubscriber {
    pub async fn new(target_accounts: Vec<Pubkey>) -> Result<Self> {
        Ok(Self {
            client: SolanaGrpcClient::from_env()?,
            target_accounts,
        })
    }

    pub async fn start(self) -> Result<()> {
        info!(
            "Starting transaction subscription for {} involved accounts",
            self.target_accounts.len()
        );

        let mut filter = TransactionFilter::new("involved_accounts");

        for account in &self.target_accounts {
            filter.account_include.push(account.to_string());
        }

        let target_accounts = self.target_accounts.clone();

        self.client
            .subscribe_transactions(
                filter,
                move |tx_update| {
                    let trace = Trace::new();

                    trace.step_with(
                        StepType::Custom("TransactionReceived".to_string()),
                        "signature",
                        &tx_update.signature,
                    );

                    let target_accounts = target_accounts.clone();
                    async move {
                        Self::handle_transaction_update(tx_update, trace, target_accounts).await
                    }
                },
                true,
            )
            .await
    }

    async fn handle_transaction_update(
        update: GrpcTransactionUpdate,
        trace: Trace,
        target_accounts: Vec<Pubkey>,
    ) -> Result<()> {
        let mut has_involved_account = false;

        if let Some(tx) = &update.transaction {
            if let Some(message) = &tx.message {
                for key_bytes in &message.account_keys {
                    if key_bytes.len() == 32 {
                        let mut array = [0u8; 32];
                        array.copy_from_slice(key_bytes);
                        let pubkey = Pubkey::from(array);
                        if target_accounts.contains(&pubkey) {
                            has_involved_account = true;
                            break;
                        }
                    }
                }
            }
        }

        if !has_involved_account {
            if let Some(meta) = &update.meta {
                for addr_bytes in &meta.loaded_writable_addresses {
                    if addr_bytes.len() == 32 {
                        let mut array = [0u8; 32];
                        array.copy_from_slice(addr_bytes);
                        let pubkey = Pubkey::from(array);
                        if target_accounts.contains(&pubkey) {
                            has_involved_account = true;
                            break;
                        }
                    }
                }

                if !has_involved_account {
                    for addr_bytes in &meta.loaded_readonly_addresses {
                        if addr_bytes.len() == 32 {
                            let mut array = [0u8; 32];
                            array.copy_from_slice(addr_bytes);
                            let pubkey = Pubkey::from(array);
                            if target_accounts.contains(&pubkey) {
                                has_involved_account = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if has_involved_account {
            trace.step_with(
                StepType::Custom("TransactionDebouncing".to_string()),
                "signature",
                &update.signature,
            );

            TRANSACTION_DEBOUNCER.update(update.signature.clone(), (update, trace));
        }

        unit_ok!()
    }
}

async fn process_involved_account_transaction(
    update: GrpcTransactionUpdate,
    trace: Trace,
) -> Result<()> {
    trace.step_with(
        StepType::Custom("ProcessingTransaction".to_string()),
        "signature",
        &update.signature,
    );
    // first, we got to figure out if it's a swap ix.
    let transaction = update.to_unified()?;
    let Some((_ix, inners)) =
        transaction.extract_ix_and_inners(|program_id| *program_id == PoolProgram::PUMP_AMM)
    else {
        return Err(anyhow!("Transaction does not contain involved accounts"));
    };

    if let Some(pool_account) = find_pump_swap_pool(&inners.instructions) {
        info!(
            "Found Pump AMM swap instruction with pool: {}",
            pool_account
        );
        if !PoolRecordRepository::is_pool_recorded(&pool_account).await {
            // this could be a new pool, we did not know, then we try to record it
            NEW_POOL_CONSUMER.publish()
        }
        // this is something we know, so trigger arbitrage opportunity.
    }

    unit_ok!()
}

fn find_pump_swap_pool(instructions: &[Instruction]) -> Option<Pubkey> {
    let pump_global_config = "ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw".to_pubkey();
    let wsol = Mints::WSOL;

    instructions.iter().find_map(|ix| {
        if ix.accounts.len() < 5 {
            return None;
        }

        if ix.accounts.get(2).map(|acc| acc.pubkey) != Some(pump_global_config) {
            return None;
        }

        let has_wsol_at_4_or_5 = ix.accounts.get(3).map(|acc| acc.pubkey) == Some(wsol)
            || ix.accounts.get(4).map(|acc| acc.pubkey) == Some(wsol);

        if has_wsol_at_4_or_5 {
            ix.accounts.get(1).map(|acc| acc.pubkey)
        } else {
            None
        }
    })
}

pub async fn start_involved_account_monitor(target_accounts: Vec<Pubkey>) -> Result<()> {
    let subscriber = InvolvedAccountSubscriber::new(target_accounts).await?;
    subscriber.start().await
}
