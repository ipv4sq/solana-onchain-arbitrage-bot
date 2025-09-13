use crate::convention::chain::util::alt::get_alt_batch;
use crate::convention::chain::util::simulation::SimulationResult;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::constant::mint::Mints;
use crate::global::daemon::blockhash::get_blockhash;
use crate::global::enums::step_type::StepType;
use crate::global::state::any_pool_holder::AnyPoolHolder;
use crate::global::trace::types::Trace;
use crate::global::wallet::get_wallet;
use crate::pipeline::uploader::common::debug;
use crate::pipeline::uploader::common::simulation_log::log_mev_simulation;
use crate::pipeline::uploader::mev_bot::construct;
use crate::pipeline::uploader::mev_bot::construct::compute_limit_ix;
use crate::pipeline::uploader::mev_bot::sender::simulate_mev_tx;
use crate::pipeline::uploader::provider::LandingChannel;
use crate::pipeline::uploader::variables::{MevBotDeduplicator, MevBotRateLimiter};
use crate::sdk::rpc::methods::transaction::compile_instruction_to_tx;
use crate::unit_ok;
use crate::util::alias::{AResult, Literal, SOLUnitLamportConvert, SOLUnitLiteralConvert};
use crate::util::env::env_config::ENV_CONFIG;
use crate::util::traits::pubkey::ToPubkey;
use construct::build_mev_ix;
use debug::print_log_to_console;
use serde_json::json;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use tracing::warn;

pub async fn fire_mev_bot(minor_mint: &Pubkey, pools: &Vec<Pubkey>, trace: Trace) -> AResult<()> {
    if !MevBotDeduplicator.can_send(minor_mint, pools) {
        warn!(
            "Duplicate transaction detected for mint {} with pools {:?}, skipping (backoff period active)",
            minor_mint,
            pools
        );
        return Ok(());
    }

    if !MevBotRateLimiter.try_acquire() {
        warn!("MEV bot rate limit exceeded, skipping execution");
        return Ok(());
    }

    trace.step_with(StepType::MevTxFired, "path", format!("{:?}", pools));
    let wallet = get_wallet();
    let configs: Vec<_> = AnyPoolHolder::batch_get(pools)
        .await
        .into_iter()
        .flatten()
        .collect();
    trace.step_with(StepType::MevTxReadyToBuild, "path", format!("{:?}", pools));
    build_and_send(
        &wallet,
        minor_mint,
        300_000,
        &configs,
        true,
        LandingChannel::HeliusSwqos,
        trace,
    )
    .await
    .map(|result| print_log_to_console(result.0, &wallet.pubkey(), result.1))?;
    unit_ok!()
}

pub async fn build_and_send(
    wallet: &Keypair,
    minor_mint: &Pubkey,
    compute_unit_limit: u32,
    pools: &[AnyPoolConfig],
    include_create_token_account_ix: bool,
    channel: LandingChannel,
    trace: Trace,
) -> AResult<(SimulationResult, Trace)> {
    trace.step(StepType::MevIxBuilding);
    let (mut instructions, _limit) = compute_limit_ix(compute_unit_limit);

    let (tip_or_unite_price_ix, tip) = channel.tip_ix(&wallet.pubkey(), 30_000);
    instructions.extend(tip_or_unite_price_ix);

    let minimum_profit = (0.00001 as Literal + tip).to_lamport();
    let mev_ix = build_mev_ix(
        wallet,
        minor_mint,
        compute_unit_limit,
        pools,
        minimum_profit,
        false,
        include_create_token_account_ix,
    )
    .await?;
    instructions.extend(mev_ix);

    let alts = get_alt_batch(&["4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey()]).await?;
    let tx = compile_instruction_to_tx(wallet, instructions, &alts, get_blockhash().await?)?;
    trace.step_with_struct(
        StepType::MevIxBuilt,
        "params",
        &json!({
            "minimum_profit": minimum_profit.to_literal(),
        }),
    );

    let simulation_result = simulate_mev_tx(&tx, &trace).await?;
    if simulation_result.err.is_none() {
        if ENV_CONFIG.enable_send_tx {
            channel.send_tx(&tx, &trace).await?;
        }
    }

    let _ = log_mev_simulation(
        &simulation_result,
        &trace,
        &wallet.pubkey(),
        &tx,
        minor_mint,
        &Mints::WSOL,
        pools,
    )
    .await;

    Ok((simulation_result, trace))
}
