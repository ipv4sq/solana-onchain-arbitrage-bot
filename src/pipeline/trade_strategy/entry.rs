use crate::database::pool_record::repository::PoolRecordRepository;
use crate::database::pool_record::PoolRecord;
use crate::dex::any_pool_config::AnyPoolConfig;
use crate::global::constant::mint::Mints;
use crate::global::enums::step_type::StepType;
use crate::global::state::any_pool_holder::AnyPoolHolder;
use crate::global::trace::types::Trace;
use crate::pipeline::event_processor::token_balance::token_balance_processor::TokenAmount;
use crate::pipeline::uploader::variables::{FireMevBotConsumer, MevBotFire};
use crate::util::alias::{MintAddress, PoolAddress};
use crate::util::structs::mint_pair::MintPair;
use futures::stream::{self, StreamExt};
use maplit::hashset;
use solana_program::pubkey::Pubkey;
use std::collections::HashSet;
use tracing::{info, trace, warn};

const MAX_OPPORTUNITIES: usize = 3;
const MAX_PROCESSING_TIME_MS: u32 = 10_000;
const WSOL_LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub async fn on_pool_update(
    pool_address: PoolAddress,
    updated_config: AnyPoolConfig,
    trace: Trace,
) -> Option<()> {
    let mints = updated_config.mint_pair();
    if !mints.contains(&Mints::WSOL) {
        return None;
    }

    let minor_mint = mints.minor_mint().ok()?;
    let blocklist = hashset! {Mints::USDC, Mints::USDT};

    if blocklist.contains(&minor_mint) {
        info!("Skipping blocklist pools");
        return None;
    }
    trace.step_with_address(StepType::TradeStrategyStarted, "pool_address", pool_address);

    if trace.since_begin() > MAX_PROCESSING_TIME_MS {
        warn!("Skipping opportunity calculation - processing time exceeded");
        return None;
    }

    trace.step_with_address(
        StepType::DetermineOpportunityStarted,
        "pool_address",
        pool_address,
    );

    let minor_mint = mints.minor_mint().unwrap();
    let opportunities =
        find_arbitrage_opportunities(&minor_mint, &pool_address, &updated_config, &trace).await?;

    if opportunities.is_empty() {
        return None;
    }

    trace.step_with(
        StepType::DetermineOpportunityFinished,
        "opportunities_count",
        opportunities.len().to_string(),
    );

    execute_opportunities(opportunities, minor_mint, trace).await;

    None
}

#[derive(Debug, Clone)]
pub struct ArbitrageResult {
    pub first_pool: PoolAddress,
    pub second_pool: PoolAddress,
    pub profit_lamports: u64,
}

async fn find_arbitrage_opportunities(
    minor_mint: &MintAddress,
    changed_pool: &PoolAddress,
    changed_config: &AnyPoolConfig,
    trace: &Trace,
) -> Option<Vec<ArbitrageResult>> {
    trace.step(StepType::DetermineOpportunityLoadingRelatedMints);

    let related_pools = load_related_pools(minor_mint, changed_pool).await?;

    trace.step_with(
        StepType::DetermineOpportunityLoadedRelatedMints,
        "amount",
        related_pools.len().to_string(),
    );

    if related_pools.is_empty() {
        return None;
    }

    trace.step_with_custom("Checking arbitrage opportunities");

    let opportunities =
        check_all_opportunities(&related_pools, changed_pool, changed_config, minor_mint).await;

    trace.step_with_custom("Completed arbitrage checks");

    select_best_opportunities(opportunities)
}

async fn load_related_pools(
    minor_mint: &MintAddress,
    exclude_pool: &PoolAddress,
) -> Option<Vec<crate::database::pool_record::model::Model>> {
    PoolRecordRepository::get_pools_contains_mint(minor_mint)
        .await
        .map(|pools| {
            pools
                .into_iter()
                .filter(|pool| {
                    pool.address.0 != *exclude_pool
                        && MintPair(pool.base_mint.0, pool.quote_mint.0)
                            .consists_of(minor_mint, &Mints::WSOL)
                            .is_ok()
                })
                .collect()
        })
}

async fn check_all_opportunities(
    related_pools: &[PoolRecord],
    changed_pool: &PoolAddress,
    changed_config: &AnyPoolConfig,
    minor_mint: &MintAddress,
) -> Vec<ArbitrageResult> {
    let input_lamports = WSOL_LAMPORTS_PER_SOL;
    let changed_pool = *changed_pool;
    let changed_config = changed_config.clone();
    let minor_mint = *minor_mint;

    stream::iter(related_pools.iter().cloned())
        .map(move |other_pool| {
            let changed_config = changed_config.clone();
            async move {
                check_bidirectional_arbitrage(
                    &other_pool,
                    &changed_pool,
                    &changed_config,
                    &minor_mint,
                    input_lamports,
                )
                .await
            }
        })
        .buffer_unordered(5)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}

async fn check_bidirectional_arbitrage(
    other_pool: &crate::database::pool_record::model::Model,
    changed_pool: &PoolAddress,
    changed_config: &AnyPoolConfig,
    minor_mint: &MintAddress,
    input_lamports: u64,
) -> Vec<ArbitrageResult> {
    let other_config = match AnyPoolHolder::get(&other_pool.address.0).await {
        Some(config) => config,
        None => return vec![],
    };

    let mut results = Vec::new();

    // Check path: changed_pool -> other_pool
    if let Some(profit) =
        simulate_arbitrage_path(input_lamports, changed_config, &other_config, minor_mint).await
    {
        if profit > 0 {
            results.push(ArbitrageResult {
                first_pool: *changed_pool,
                second_pool: other_pool.address.0,
                profit_lamports: profit,
            });
        }
    }

    // Check path: other_pool -> changed_pool
    if let Some(profit) =
        simulate_arbitrage_path(input_lamports, &other_config, changed_config, minor_mint).await
    {
        if profit > 0 {
            results.push(ArbitrageResult {
                first_pool: other_pool.address.0,
                second_pool: *changed_pool,
                profit_lamports: profit,
            });
        }
    }

    results
}

#[allow(dead_code)]
struct PoolDepthAnalysis {
    base_reserve: TokenAmount,
    quote_reserve: TokenAmount,
    wsol_reserve: TokenAmount,
    minor_reserve: TokenAmount,
    max_base_trade: TokenAmount,
    max_quote_trade: TokenAmount,
}

async fn analyze_pool_depth(
    config: &AnyPoolConfig,
    minor_mint: &MintAddress,
    impact_threshold: f64,
) -> Option<PoolDepthAnalysis> {
    const MIN_RESERVE_SOL: f64 = 0.01; // 0.01 SOL minimum

    let (base_token_amount, quote_token_amount) = config.get_reserves().await;

    let base_reserve = base_token_amount?;
    let quote_reserve = quote_token_amount?;

    // Determine which reserves correspond to WSOL and minor token
    let wsol_reserve = if config.base_mint() == Mints::WSOL {
        base_reserve.clone()
    } else {
        quote_reserve.clone()
    };
    
    let minor_reserve = if config.base_mint() == *minor_mint {
        base_reserve.clone()
    } else {
        quote_reserve.clone()
    };

    // Convert WSOL reserve to SOL value
    let wsol_value = wsol_reserve.to_value();
    
    // For minor token, we just check if it has any liquidity
    let minor_value = minor_reserve.to_value();

    if wsol_value < MIN_RESERVE_SOL || minor_value == 0.0 {
        trace!(
            "Pool {} insufficient liquidity: WSOL={} SOL, minor={} (decimals={})",
            config.pool_address(),
            wsol_value,
            minor_value,
            minor_reserve.decimals
        );
        return None;
    }

    // Calculate max trades based on impact threshold
    let max_base_trade = TokenAmount {
        amount: (base_reserve.amount as f64 * impact_threshold * 2.0) as u64,
        decimals: base_reserve.decimals,
    };
    
    let max_quote_trade = TokenAmount {
        amount: (quote_reserve.amount as f64 * impact_threshold * 2.0) as u64,
        decimals: quote_reserve.decimals,
    };

    Some(PoolDepthAnalysis {
        base_reserve,
        quote_reserve,
        wsol_reserve,
        minor_reserve,
        max_base_trade,
        max_quote_trade,
    })
}

async fn simulate_arbitrage_path(
    input_sol_lamports: u64,
    first_config: &AnyPoolConfig,
    second_config: &AnyPoolConfig,
    minor_mint: &MintAddress,
) -> Option<u64> {
    const IMPACT_THRESHOLD: f64 = 0.005;

    // Analyze both pools' depth in parallel
    let (pool1_depth, pool2_depth) = tokio::join!(
        analyze_pool_depth(first_config, minor_mint, IMPACT_THRESHOLD),
        analyze_pool_depth(second_config, minor_mint, IMPACT_THRESHOLD)
    );

    let pool1_depth = pool1_depth?;
    let pool2_depth = pool2_depth?;

    // Determine safe amount for first pool (get the amount value)
    let safe_input_for_first = if first_config.base_mint() == Mints::WSOL {
        pool1_depth.max_base_trade.amount
    } else {
        pool1_depth.max_quote_trade.amount
    };

    // Adjust input amount if it exceeds safe threshold
    let adjusted_input = input_sol_lamports.min(safe_input_for_first);

    if adjusted_input < input_sol_lamports {
        trace!(
            "Adjusted input from {} to {} due to liquidity constraints",
            input_sol_lamports,
            adjusted_input
        );
    }

    // First swap: WSOL -> minor_mint with adjusted amount
    let tokens_received =
        simulate_swap(adjusted_input, first_config, &Mints::WSOL, minor_mint).await?;

    // Check if second pool can handle the output from first pool (get the amount value)
    let safe_input_for_second = if second_config.base_mint() == *minor_mint {
        pool2_depth.max_base_trade.amount
    } else {
        pool2_depth.max_quote_trade.amount
    };

    // Adjust tokens if needed
    let adjusted_tokens = tokens_received.min(safe_input_for_second);

    // Second swap: minor_mint -> WSOL with adjusted amount
    let output_sol =
        simulate_swap(adjusted_tokens, second_config, minor_mint, &Mints::WSOL).await?;

    // Calculate profit (can be negative if unprofitable)
    Some(output_sol.saturating_sub(adjusted_input))
}

async fn simulate_swap(
    input_amount: u64,
    config: &AnyPoolConfig,
    from_mint: &Pubkey,
    to_mint: &Pubkey,
) -> Option<u64> {
    config
        .get_amount_out(input_amount, from_mint, to_mint)
        .await
        .ok()
}

fn select_best_opportunities(
    mut opportunities: Vec<ArbitrageResult>,
) -> Option<Vec<ArbitrageResult>> {
    if opportunities.is_empty() {
        return None;
    }

    // Sort by profit descending
    opportunities.sort_by(|a, b| b.profit_lamports.cmp(&a.profit_lamports));

    // Deduplicate pool pairs and take top opportunities
    let mut seen_pairs = HashSet::new();
    let unique_opportunities: Vec<_> = opportunities
        .into_iter()
        .filter(|result| {
            let pair = (result.first_pool, result.second_pool);
            seen_pairs.insert(pair)
        })
        .take(MAX_OPPORTUNITIES)
        .collect();

    if unique_opportunities.is_empty() {
        None
    } else {
        Some(unique_opportunities)
    }
}

async fn execute_opportunities(
    opportunities: Vec<ArbitrageResult>,
    minor_mint: MintAddress,
    trace: Trace,
) {
    for (i, opportunity) in opportunities.iter().enumerate() {
        let pools_for_mev = vec![opportunity.first_pool, opportunity.second_pool];

        info!(
            "🚀 MEV Opportunity #{}: {} -> {} (profit: {} SOL)",
            i + 1,
            opportunity.first_pool,
            opportunity.second_pool,
            opportunity.profit_lamports as f64 / WSOL_LAMPORTS_PER_SOL as f64,
        );

        trace.step_with(
            StepType::MevTxTryToFile,
            "path",
            format!("{:?}", pools_for_mev),
        );

        let _ = FireMevBotConsumer
            .publish(MevBotFire {
                minor_mint,
                pools: pools_for_mev,
                trace: trace.clone(),
            })
            .await;
    }
}
