use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::pool_do::Model as PoolRecord;
use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::trace::types::{StepType, Trace};
use crate::arb::pipeline::swap_changes::account_monitor::pool_tracker::get_minor_mint_for_pool;
use crate::arb::pipeline::swap_changes::account_monitor::pool_update::PoolUpdate;
use crate::arb::pipeline::swap_changes::cache::PoolConfigCache;
use crate::arb::pipeline::trade_strategy::price_tracker::{
    clear_prices_for_token, detect_arbitrage, update_pool_prices, ArbitrageOpportunity,
};
use crate::arb::pipeline::uploader::entry::{FireMevBotConsumer, MevBotFire};
use crate::arb::util::alias::PoolAddress;
use crate::arb::util::alias::{AResult, MintAddress};
use rust_decimal::Decimal;
use solana_program::pubkey::Pubkey;
use tracing::info;

pub async fn on_pool_update(update: PoolUpdate, trace: Trace) -> Option<()> {
    let pool_address: PoolAddress = update.pool().clone();
    trace.step_with_address(StepType::TradeStrategyStarted, "pool_address", pool_address);
    // update pool
    let updated_config = AnyPoolConfig::from_account_update(&update.current, &Mints::WSOL)
        .await
        .ok()?;
    PoolConfigCache.put(pool_address, updated_config).await;

    info!("🔍 on_pool_update triggered for pool: {}", pool_address);

    let mint = get_minor_mint_for_pool(&pool_address).await?;
    info!(
        "📊 Pool update detected - Pool: {}, Mint: {}",
        pool_address, mint
    );

    let pool_records = PoolRecordRepository::get_pools_contains_mint(&mint).await?;
    info!(
        "Found {} pool records for mint {}",
        pool_records.len(),
        mint
    );

    let updated_pool_record = pool_records.iter().find(|p| p.address.0 == pool_address)?;
    info!(
        "Processing pool update: {} (DEX: {:?})",
        pool_address, updated_pool_record.dex_type
    );
    trace.step_with_address(
        StepType::DetermineOpportunityStarted,
        "pool_address",
        pool_address,
    );
    if let Some(opportunity) = compute(&mint, updated_pool_record).await {
        trace.step_with(
            StepType::DetermineOpportunityFinished,
            "spread",
            opportunity.spread.to_string(),
        );
        let pools_for_mev = vec![opportunity.buy_pool, opportunity.sell_pool];
        info!("🚀 Try to MEV bot fire for pools: {:?}", pools_for_mev);
        trace.step_with(
            StepType::MevTxTryToFile,
            "path",
            format!("{:?}", pools_for_mev),
        );
        let _ = FireMevBotConsumer
            .publish(MevBotFire {
                minor_mint: mint,
                pools: pools_for_mev,
                trace,
            })
            .await;
    } else {
        info!("No arbitrage opportunity found for mint {}", mint);
    }

    None
}

pub async fn compute(
    minor_mint: &MintAddress,
    _updated_pool: &PoolRecord,
) -> Option<ArbitrageOpportunity> {
    info!("🔧 Computing arbitrage for mint: {}", minor_mint);

    let related_pools = PoolRecordRepository::get_pools_contains_mint(minor_mint).await?;
    info!(
        "Found {} related pools for arbitrage computation",
        related_pools.len()
    );

    if related_pools.len() < 2 {
        info!(
            "Insufficient pools (< 2) for arbitrage on mint {}",
            minor_mint
        );
        return None;
    }

    clear_prices_for_token(*minor_mint);
    info!("Cleared price cache for token {}", minor_mint);

    for pool in related_pools.iter() {
        let config = PoolConfigCache.get(&pool.address.0).await?;
        info!(
            "Processing pool {} (DEX: {:?})",
            pool.address.0, pool.dex_type
        );

        if let Some((buy_price, sell_price)) =
            calculate_pool_prices(config, *minor_mint, &Mints::WSOL).await
        {
            info!(
                "Pool {} prices - Buy: {}, Sell: {}",
                pool.address.0, buy_price, sell_price
            );
            update_pool_prices(*minor_mint, pool.address.0, buy_price, sell_price);
        } else {
            info!("Failed to calculate prices for pool {}", pool.address.0);
        }
    }

    let opportunity = detect_arbitrage(*minor_mint);

    if let Some(ref arb) = opportunity {
        info!(
            "🎯 Arbitrage Opportunity Detected!
            Token: {:?}
            Buy from pool: {} @ {} SOL
            Sell to pool: {} @ {} SOL
            Spread: {} SOL
            Profit percentage: {:.2}%",
            arb.token_mint,
            arb.buy_pool,
            arb.buy_price,
            arb.sell_pool,
            arb.sell_price,
            arb.spread,
            arb.spread / arb.buy_price * Decimal::from(100)
        );
    }

    opportunity
}

async fn calculate_pool_prices(
    config: AnyPoolConfig,
    token_mint: MintAddress,
    sol_mint: &Pubkey,
) -> Option<(Decimal, Decimal)> {
    match config {
        AnyPoolConfig::MeteoraDlmm(ref c) => {
            info!("Calculating prices for MeteoraDlmm pool");
            let sol_to_token = c
                .data
                .mid_price_for_quick_estimate(sol_mint, &token_mint)
                .await
                .ok()?;
            let token_to_sol = c
                .data
                .mid_price_for_quick_estimate(&token_mint, sol_mint)
                .await
                .ok()?;

            info!(
                "Raw prices - sol_to_token: {} tokens per SOL, token_to_sol: {} SOL per token",
                sol_to_token.mid_price, token_to_sol.mid_price
            );

            let buy_price = Decimal::ONE / sol_to_token.mid_price;
            let sell_price = token_to_sol.mid_price;

            info!(
                "MeteoraDlmm prices calculated - Buy: {} SOL per token, Sell: {} SOL per token",
                buy_price, sell_price
            );
            Some((buy_price, sell_price))
        }
        AnyPoolConfig::MeteoraDammV2(ref c) => {
            info!("Calculating prices for MeteoraDammV2 pool");
            let sol_to_token = c
                .data
                .mid_price_for_quick_estimate(sol_mint, &token_mint)
                .await
                .ok()?;
            let token_to_sol = c
                .data
                .mid_price_for_quick_estimate(&token_mint, sol_mint)
                .await
                .ok()?;

            info!(
                "Raw prices - sol_to_token: {} tokens per SOL, token_to_sol: {} SOL per token",
                sol_to_token.mid_price, token_to_sol.mid_price
            );

            let buy_price = Decimal::ONE / sol_to_token.mid_price;
            let sell_price = token_to_sol.mid_price;

            info!(
                "MeteoraDammV2 prices calculated - Buy: {} SOL per token, Sell: {} SOL per token",
                buy_price, sell_price
            );
            Some((buy_price, sell_price))
        }
        AnyPoolConfig::Unsupported => {
            info!("Unsupported pool config type");
            None
        }
    }
}
