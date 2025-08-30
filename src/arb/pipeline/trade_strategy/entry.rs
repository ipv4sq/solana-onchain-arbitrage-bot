use crate::arb::database::pool_record::repository::PoolRecordRepository;
use crate::arb::dex::any_pool_config::AnyPoolConfig;
use crate::arb::dex::any_pool_config::PoolConfigCache;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::enums::step_type::StepType;
use crate::arb::global::trace::types::Trace;
use crate::arb::pipeline::event_processor::pool_update_processor::get_minor_mint_for_pool;
use crate::arb::pipeline::uploader::variables::{FireMevBotConsumer, MevBotFire};
use crate::arb::util::alias::MintAddress;
use crate::arb::util::alias::PoolAddress;
use crate::arb::util::structs::mint_pair::MintPair;
use rust_decimal::Decimal;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use tracing::{info, warn};

pub async fn on_pool_update(
    pool_address: PoolAddress,
    updated_config: AnyPoolConfig,
    trace: Trace,
) -> Option<()> {
    trace.step_with_address(StepType::TradeStrategyStarted, "pool_address", pool_address);

    PoolConfigCache
        .put(pool_address, updated_config.clone())
        .await;

    info!("🔍 on_pool_update triggered for pool: {}", pool_address);

    let mint = get_minor_mint_for_pool(&pool_address).await?;

    if trace.since_begin() > 10_000 {
        // not do anything if it's buffered too long, so we can quickly catch up.
        warn!("skipping oppotunity calculation because it's too long");
        return None;
    }

    trace.step_with_address(
        StepType::DetermineOpportunityStarted,
        "pool_address",
        pool_address,
    );

    if let Some(opportunities) =
        compute_brute_force(&mint, &pool_address, &updated_config, &trace).await
    {
        if !opportunities.is_empty() {
            trace.step_with(
                StepType::DetermineOpportunityFinished,
                "opportunities_count",
                opportunities.len().to_string(),
            );

            for (i, opportunity) in opportunities.iter().enumerate() {
                let pools_for_mev = vec![opportunity.first_pool, opportunity.second_pool];
                info!(
                    "🚀 MEV Opportunity #{}: First: {}, Second: {}, Output: {} SOL",
                    i + 1,
                    opportunity.first_pool,
                    opportunity.second_pool,
                    opportunity.output_sol
                );
                trace.step_with(
                    StepType::MevTxTryToFile,
                    "path",
                    format!("{:?}", pools_for_mev),
                );
                let _ = FireMevBotConsumer
                    .publish(MevBotFire {
                        minor_mint: mint,
                        pools: pools_for_mev,
                        trace: trace.clone(),
                    })
                    .await;
            }
        }
    } else {
        info!("No arbitrage opportunity found for mint {}", mint);
    }

    None
}

#[derive(Debug, Clone)]
pub struct ArbitrageResult {
    pub first_pool: PoolAddress,
    pub second_pool: PoolAddress,
    pub output_sol: Decimal,
}

pub async fn compute_brute_force(
    minor_mint: &MintAddress,
    changed_pool: &PoolAddress,
    changed_config: &AnyPoolConfig,
    trace: &Trace,
) -> Option<Vec<ArbitrageResult>> {
    info!(
        "🔧 Computing brute force arbitrage for mint: {}",
        minor_mint
    );
    trace.step(StepType::DetermineOpportunityLoadingRelatedMints);

    let related_pools = PoolRecordRepository::get_pools_contains_mint(minor_mint)
        .await?
        .into_iter()
        .filter(|pool| pool.address.0 != *changed_pool)
        .filter(|pool| {
            let mint_pair = MintPair(pool.base_mint.0, pool.quote_mint.0);
            mint_pair.consists_of(minor_mint, &Mints::WSOL).is_ok()
        })
        .collect::<Vec<_>>();

    trace.step_with(
        StepType::DetermineOpportunityLoadedRelatedMints,
        "amount",
        related_pools.len().to_string(),
    );

    info!(
        "Found {} other pools for brute force with changed pool {}",
        related_pools.len(),
        changed_pool
    );

    if related_pools.is_empty() {
        info!("No other pools found for arbitrage on mint {}", minor_mint);
        return None;
    }

    let input_sol = Decimal::ONE;
    let mut results = Vec::new();

    trace.step_with_custom("Checking arbitrage of each pool");

    for other_pool in related_pools.iter() {
        check_pool_arbitrage(
            &mut results,
            other_pool,
            changed_pool,
            changed_config,
            *minor_mint,
            input_sol,
        )
        .await;
    }

    trace.step_with_custom("Checked arbitrage of each pool");

    results.sort_by(|a, b| b.output_sol.cmp(&a.output_sol));

    let mut unique_pairs = HashSet::new();
    let mut unique_results = Vec::new();

    for result in results {
        let pair = (result.first_pool, result.second_pool);
        if unique_pairs.insert(pair) && unique_results.len() < 3 {
            info!(
                "🎯 Top opportunity: {} -> {} = {} SOL (profit: {} SOL)",
                result.first_pool,
                result.second_pool,
                result.output_sol,
                result.output_sol - Decimal::ONE
            );
            unique_results.push(result);
        }
    }

    if unique_results.is_empty() {
        None
    } else {
        Some(unique_results)
    }
}

async fn check_pool_arbitrage(
    results: &mut Vec<ArbitrageResult>,
    other_pool: &crate::arb::database::pool_record::model::Model,
    changed_pool: &PoolAddress,
    changed_config: &AnyPoolConfig,
    minor_mint: MintAddress,
    input_sol: Decimal,
) {
    let other_config = match PoolConfigCache.get(&other_pool.address.0).await {
        Some(config) => config,
        None => {
            info!(
                "Failed to get config for pool {}, skipping",
                other_pool.address.0
            );
            return;
        }
    };

    if let Some(output_sol) =
        simulate_path(input_sol, changed_config, &other_config, minor_mint).await
    {
        if output_sol > Decimal::ONE {
            results.push(ArbitrageResult {
                first_pool: *changed_pool,
                second_pool: other_pool.address.0,
                output_sol,
            });
            info!(
                "✅ Scenario 1 profitable: {} -> {} = {} SOL",
                changed_pool, other_pool.address.0, output_sol
            );
        }
    }

    if let Some(output_sol) =
        simulate_path(input_sol, &other_config, changed_config, minor_mint).await
    {
        if output_sol > Decimal::ONE {
            results.push(ArbitrageResult {
                first_pool: other_pool.address.0,
                second_pool: *changed_pool,
                output_sol,
            });
            info!(
                "✅ Scenario 2 profitable: {} -> {} = {} SOL",
                other_pool.address.0, changed_pool, output_sol
            );
        }
    }
}

async fn simulate_path(
    input_sol: Decimal,
    first_config: &AnyPoolConfig,
    second_config: &AnyPoolConfig,
    minor_mint: MintAddress,
) -> Option<Decimal> {
    let tokens_received = simulate_swap(input_sol, first_config, &Mints::WSOL, &minor_mint).await?;

    let output_sol =
        simulate_swap(tokens_received, second_config, &minor_mint, &Mints::WSOL).await?;

    Some(output_sol)
}

async fn simulate_swap(
    input_amount: Decimal,
    config: &AnyPoolConfig,
    from_mint: &Pubkey,
    to_mint: &Pubkey,
) -> Option<Decimal> {
    let quote = config.mid_price(from_mint, to_mint).await.ok()?;
    Some(input_amount * quote.mid_price)
}
