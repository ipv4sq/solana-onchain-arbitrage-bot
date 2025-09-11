use crate::convention::chain::util::simulation::SimulationResult;
use crate::database::mev_simulation_log::repository::MevSimulationLogRepository;
use crate::database::mev_simulation_log::{
    MevSimulationLogDetails, MevSimulationLogParams, SimulationAccount,
};
use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::trace::types::Trace;
use crate::util::solana::pda::ata;
use solana_program::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use tracing::error;

pub async fn log_mev_simulation(
    result: &SimulationResult,
    trace: &Trace,
    owner: &Pubkey,
    tx: &VersionedTransaction,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    pools: &[AnyPoolConfig],
) -> anyhow::Result<()> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();

    let simulation_status = if result.err.is_some() {
        "failed"
    } else {
        "success"
    };

    let error_message = result.err.clone();
    let logs = Some(result.logs.clone());
    let compute_units_consumed = result.units_consumed.map(|u| u as i64);

    let return_data = if let Some(ref _meta) = result.meta {
        // Extract return data from meta if available
        None // TransactionMeta doesn't have return_data field based on the struct
    } else {
        None
    };

    let pool_addresses: Vec<String> = pools.iter().map(|p| p.pool_address().to_string()).collect();

    let pool_types: Vec<String> = pools.iter().map(|p| p.dex_type().to_string()).collect();

    let minor_mint_record = MintRecordRepository::get_mint_or_err(minor_mint).await?;
    let desired_mint_record = MintRecordRepository::get_mint_or_err(desired_mint).await?;

    // Calculate actual profit from simulation results
    // Find the user's token account for the desired mint
    let user_ata = ata(owner, desired_mint, &spl_token::ID);
    let user_ata_str = user_ata.to_string();

    // When simulation fails or doesn't return metadata, we can't calculate actual profit
    // Mark both profitable and profitability as None to distinguish from actual 0 values
    let (profitable, profitability) = if let Some(ref meta) = result.meta {
        // Find the token balance changes for the user's ATA
        let actual_profit = meta
            .post_token_balances
            .iter()
            .find(|tb| {
                tb.mint == desired_mint.to_string() && tb.owner.as_ref() == Some(&user_ata_str)
            })
            .and_then(|post| {
                meta.pre_token_balances
                    .iter()
                    .find(|pre| {
                        pre.mint == desired_mint.to_string()
                            && pre.account_index == post.account_index
                    })
                    .map(|pre| {
                        // Parse as i128 first to avoid overflow, then check if it fits in i64
                        let post_amount: i128 = post.ui_token_amount.amount.parse().unwrap_or(0);
                        let pre_amount: i128 = pre.ui_token_amount.amount.parse().unwrap_or(0);
                        let profit = post_amount - pre_amount;

                        // Clamp to i64 range if overflow would occur
                        if profit > i64::MAX as i128 {
                            i64::MAX
                        } else if profit < i64::MIN as i128 {
                            i64::MIN
                        } else {
                            profit as i64
                        }
                    })
            })
            .unwrap_or(0);

        // Set profitable based on actual profit value (positive, zero, or negative)
        // Only when simulation succeeded
        let is_profitable = if simulation_status == "success" {
            Some(actual_profit > 0)
        } else {
            // Failed simulations are not profitable
            Some(false)
        };

        // Always record the actual profit/loss value (can be negative)
        (is_profitable, Some(actual_profit))
    } else {
        // No meta available - can't determine profitability
        (None, None)
    };

    // Extract accounts from the transaction
    let accounts: Vec<SimulationAccount> = {
        let (account_keys, header) = match &tx.message {
            solana_sdk::message::VersionedMessage::Legacy(msg) => (&msg.account_keys, &msg.header),
            solana_sdk::message::VersionedMessage::V0(msg) => (&msg.account_keys, &msg.header),
        };

        account_keys
            .iter()
            .enumerate()
            .map(|(idx, pubkey)| {
                let is_signer = idx < header.num_required_signatures as usize;
                let is_writable = if is_signer {
                    idx < (header.num_required_signatures - header.num_readonly_signed_accounts)
                        as usize
                } else {
                    let non_signer_idx = idx - header.num_required_signatures as usize;
                    let num_writable_unsigned = account_keys.len()
                        - header.num_required_signatures as usize
                        - header.num_readonly_unsigned_accounts as usize;
                    non_signer_idx < num_writable_unsigned
                };
                SimulationAccount {
                    pubkey: *pubkey,
                    is_signer,
                    is_writable,
                }
            })
            .collect()
    };

    let params = MevSimulationLogParams {
        minor_mint: *minor_mint,
        desired_mint: *desired_mint,
        minor_mint_sym: minor_mint_record.repr,
        desired_mint_sym: desired_mint_record.repr,
        pools: pool_addresses,
        pool_types,
        profitable,
        profitability,
        details: MevSimulationLogDetails { accounts },
        tx_size: Some(tx_size as i32),
        simulation_status: Some(simulation_status.to_string()),
        compute_units_consumed,
        error_message,
        logs,
        return_data,
        units_per_byte: None,
        trace: Some(trace.dump_json()),
        reason: generate_reason(result),
    };

    if let Err(e) = MevSimulationLogRepository::insert(params).await {
        error!("Failed to log MEV simulation: {}", e);
    }

    Ok(())
}

fn generate_reason(result: &SimulationResult) -> Option<String> {
    for log in &result.logs {
        let log_lower = log.to_lowercase();

        if log_lower.contains("no profitable") {
            return Some("No profitable path".to_string());
        }

        if log_lower.contains("insufficient funds") {
            return Some("Insufficient funds".to_string());
        }
    }

    None
}
