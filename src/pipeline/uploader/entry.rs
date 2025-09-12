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
use crate::pipeline::uploader::mev_bot::sender::simulate_mev_tx;
use crate::pipeline::uploader::provider::jito::get_jito_tips;
use crate::pipeline::uploader::provider::sender::send_real_mev_tx;
use crate::pipeline::uploader::provider::SenderChannel;
use crate::pipeline::uploader::variables::{MevBotDeduplicator, MevBotRateLimiter, ENABLE_SEND_TX};
use crate::sdk::rpc::methods::transaction::compile_instruction_to_tx;
use crate::unit_ok;
use crate::util::alias::AResult;
use crate::util::traits::pubkey::ToPubkey;
use construct::build_mev_ix;
use debug::print_log_to_console;
use futures::future::join_all;
use solana_program::pubkey::Pubkey;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
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
    let configs: Vec<_> = join_all(
        pools
            .iter()
            .map(|pool_address| async move { AnyPoolHolder::get(pool_address).await }),
    )
    .await
    .into_iter()
    .flatten()
    .collect();
    trace.step_with(StepType::MevTxReadyToBuild, "path", format!("{:?}", pools));

    let wallet_pubkey = wallet.pubkey();
    let jito_tips = get_jito_tips()
        .map(|t| t.landed_tips_75th_percentile)
        .unwrap_or(0.00001);
    let minimum_profit = jito_tips + 0.0001;
    build_and_send(
        &wallet,    //
        minor_mint, //
        250_000,    //
        // 30_000,
        1,                                                 //
        &configs,                                          //
        (minimum_profit * LAMPORTS_PER_SOL as f64) as u64, //
        true,                                              //
        trace,
    )
    .await
    .map(|result| print_log_to_console(result.0, &wallet_pubkey, result.1))?;
    unit_ok!()
}

pub async fn build_and_send(
    wallet: &Keypair,
    minor_mint: &Pubkey,
    compute_unit_limit: u32,
    unit_price: u64,
    pools: &[AnyPoolConfig],
    minimum_profit: u64,
    include_create_token_account_ix: bool,
    trace: Trace,
) -> anyhow::Result<(SimulationResult, Trace)> {
    trace.step(StepType::MevIxBuilding);
    let alts = get_alt_batch(&["4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey()]).await?;

    let mev_ix = build_mev_ix(
        wallet,
        minor_mint,
        compute_unit_limit,
        unit_price,
        pools,
        minimum_profit,
        false,
        include_create_token_account_ix,
    )
    .await?;

    let tx = compile_instruction_to_tx(
        wallet,
        mev_ix,
        &alts,
        get_blockhash().await?, //compile the tx
    )?;

    trace.step(StepType::MevIxBuilt);

    let simulation_result = simulate_mev_tx(&tx, &trace).await?;

    if simulation_result.err.is_none() {
        // alright, let's get it
        if *ENABLE_SEND_TX {
            // let new_tx = build_tx(
            //     wallet,
            //     minor_mint,
            //     compute_unit_limit,
            //     unit_price,
            //     pools,
            //     get_blockhash().await?,
            //     &alts,
            //     minimum_profit,
            //     false,
            //     include_create_token_account_ix,
            // )
            // .await?;
            trace.step(StepType::MevRealTxBuilding);
            send_real_mev_tx(&tx, &trace).await?;
            // sender(&tx).await?
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
