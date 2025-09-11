use crate::convention::chain::meta::TransactionMeta;
use crate::convention::chain::util::simulation::SimulationResult;
use crate::database::mev_simulation_log::repository::MevSimulationLogRepository;
use crate::database::mev_simulation_log::{
    MevSimulationLogDetails, MevSimulationLogParams, SimulationAccount,
};
use crate::database::mint_record::repository::MintRecordRepository;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::trace::types::Trace;
use crate::util::alias::AResult;
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
) -> AResult<()> {
    let tx_size = bincode::serialize(tx)?.len() as i32;
    let simulation_status = get_simulation_status(result);

    let (profitable, profitability) =
        calculate_profitability(result, owner, desired_mint, &simulation_status);

    let accounts = extract_transaction_accounts(tx);

    let (minor_mint_sym, desired_mint_sym) = fetch_mint_symbols(minor_mint, desired_mint).await?;

    let params = build_simulation_log_params(
        result,
        trace,
        minor_mint,
        desired_mint,
        minor_mint_sym,
        desired_mint_sym,
        pools,
        profitable,
        profitability,
        accounts,
        tx_size,
        simulation_status,
    );

    if let Err(e) = MevSimulationLogRepository::insert(params).await {
        error!("Failed to log MEV simulation: {}", e);
    }

    Ok(())
}

fn get_simulation_status(result: &SimulationResult) -> &'static str {
    if result.err.is_some() {
        "failed"
    } else {
        "success"
    }
}

fn calculate_profitability(
    result: &SimulationResult,
    owner: &Pubkey,
    desired_mint: &Pubkey,
    simulation_status: &str,
) -> (Option<bool>, Option<i64>) {
    let Some(ref meta) = result.meta else {
        return (None, None);
    };

    let user_ata = ata(owner, desired_mint, &spl_token::ID);
    let user_ata_str = user_ata.to_string();

    let actual_profit = calculate_token_balance_change(meta, desired_mint, &user_ata_str);

    let is_profitable = if simulation_status == "success" {
        Some(actual_profit > 0)
    } else {
        Some(false)
    };

    (is_profitable, Some(actual_profit))
}

fn calculate_token_balance_change(
    meta: &TransactionMeta,
    desired_mint: &Pubkey,
    user_ata_str: &str,
) -> i64 {
    meta.post_token_balances
        .iter()
        .find(|tb| {
            tb.mint == desired_mint.to_string()
                && tb.owner.as_ref() == Some(&user_ata_str.to_string())
        })
        .and_then(|post| {
            meta.pre_token_balances
                .iter()
                .find(|pre| {
                    pre.mint == desired_mint.to_string() && pre.account_index == post.account_index
                })
                .map(|pre| {
                    let post_amount: i128 = post.ui_token_amount.amount.parse().unwrap_or(0);
                    let pre_amount: i128 = pre.ui_token_amount.amount.parse().unwrap_or(0);
                    let profit = post_amount - pre_amount;

                    profit.clamp(i64::MIN as i128, i64::MAX as i128) as i64
                })
        })
        .unwrap_or(0)
}

fn extract_transaction_accounts(tx: &VersionedTransaction) -> Vec<SimulationAccount> {
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
}

async fn fetch_mint_symbols(
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
) -> anyhow::Result<(String, String)> {
    let minor_mint_record = MintRecordRepository::get_mint_or_err(minor_mint).await?;
    let desired_mint_record = MintRecordRepository::get_mint_or_err(desired_mint).await?;
    Ok((minor_mint_record.repr, desired_mint_record.repr))
}

fn build_simulation_log_params(
    result: &SimulationResult,
    trace: &Trace,
    minor_mint: &Pubkey,
    desired_mint: &Pubkey,
    minor_mint_sym: String,
    desired_mint_sym: String,
    pools: &[AnyPoolConfig],
    profitable: Option<bool>,
    profitability: Option<i64>,
    accounts: Vec<SimulationAccount>,
    tx_size: i32,
    simulation_status: &str,
) -> MevSimulationLogParams {
    MevSimulationLogParams {
        minor_mint: *minor_mint,
        desired_mint: *desired_mint,
        minor_mint_sym,
        desired_mint_sym,
        pools: pools.iter().map(|p| p.pool_address().to_string()).collect(),
        pool_types: pools.iter().map(|p| p.dex_type().to_string()).collect(),
        profitable,
        profitability,
        details: MevSimulationLogDetails { accounts },
        tx_size: Some(tx_size),
        simulation_status: Some(simulation_status.to_string()),
        compute_units_consumed: result.units_consumed.map(|u| u as i64),
        error_message: result.err.clone(),
        logs: Some(result.logs.clone()),
        return_data: None,
        units_per_byte: None,
        trace: Some(trace.dump_json()),
        reason: generate_reason(result),
    }
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
