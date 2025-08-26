use crate::arb::convention::pool::register::AnyPoolConfig;
use crate::arb::database::entity::pool_do::Model as PoolRecord;
use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
use crate::arb::global::constant::mint::Mints;
use crate::arb::pipeline::swap_changes::account_monitor::pool_vault::get_mint_and_pool_of_vault;
use crate::arb::pipeline::swap_changes::account_monitor::vault_update::VaultUpdate;
use crate::arb::pipeline::swap_changes::cache::PoolConfigCache;
use crate::arb::pipeline::trade_strategy::price_tracker::{
    ArbitrageOpportunity, clear_prices_for_token, detect_arbitrage, update_pool_prices
};
use crate::arb::pipeline::uploader::entry::{FireMevBotConsumer, MevBotFire};
use crate::arb::util::alias::MintAddress;
use rust_decimal::Decimal;
use solana_program::pubkey::Pubkey;
use tracing::{info, debug};

pub async fn on_swap_occurred(update: VaultUpdate) -> Option<()> {
    let vault = update.current.pubkey;
    debug!("🔍 on_swap_occurred triggered for vault: {}", vault);
    
    let (mint, pool) = get_mint_and_pool_of_vault(&vault)?;
    info!("📊 Swap detected - Vault: {}, Mint: {}, Pool: {}", vault, mint, pool);
    
    let pool_records = PoolRecordRepository::get_pools(&mint).await?;
    debug!("Found {} pool records for mint {}", pool_records.len(), mint);
    
    let updated_pool_record = pool_records.iter().find(|p| p.address.0 == pool)?;
    info!("Processing pool update: {} (DEX: {:?})", pool, updated_pool_record.dex_type);
    
    if let Some(opportunity) = compute(&mint, updated_pool_record).await {
        info!(
            "✅ Found arbitrage opportunity with spread {} SOL for mint {}",
            opportunity.spread, mint
        );
        
        let pools_for_mev = vec![opportunity.buy_pool, opportunity.sell_pool];
        
        info!("🚀 Publishing MEV bot fire for pools: {:?}", pools_for_mev);
        let _ = FireMevBotConsumer
            .publish(MevBotFire {
                minor_mint: mint,
                pools: pools_for_mev,
            })
            .await;
    } else {
        debug!("No arbitrage opportunity found for mint {}", mint);
    }
    
    None
}

pub async fn compute(minor_mint: &MintAddress, _updated_pool: &PoolRecord) -> Option<ArbitrageOpportunity> {
    debug!("🔧 Computing arbitrage for mint: {}", minor_mint);
    
    let related_pools = PoolRecordRepository::get_pools(minor_mint).await?;
    info!("Found {} related pools for arbitrage computation", related_pools.len());
    
    if related_pools.len() < 2 {
        debug!("Insufficient pools (< 2) for arbitrage on mint {}", minor_mint);
        return None;
    }
    
    clear_prices_for_token(*minor_mint);
    debug!("Cleared price cache for token {}", minor_mint);
    
    for pool in related_pools.iter() {
        let config = PoolConfigCache.get(&pool.address.0).await?;
        debug!("Processing pool {} (DEX: {:?})", pool.address.0, pool.dex_type);
        
        if let Some((buy_price, sell_price)) = calculate_pool_prices(config, *minor_mint, &Mints::WSOL).await {
            debug!(
                "Pool {} prices - Buy: {}, Sell: {}",
                pool.address.0, buy_price, sell_price
            );
            update_pool_prices(
                *minor_mint,
                pool.address.0,
                buy_price,
                sell_price,
            );
        } else {
            debug!("Failed to calculate prices for pool {}", pool.address.0);
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
            debug!("Calculating prices for MeteoraDlmm pool");
            let sol_to_token = c.data
                .mid_price_for_quick_estimate(sol_mint, &token_mint)
                .await
                .ok()?;
            let token_to_sol = c.data
                .mid_price_for_quick_estimate(&token_mint, sol_mint)
                .await
                .ok()?;
            
            let buy_price = Decimal::ONE / sol_to_token.mid_price;
            let sell_price = token_to_sol.mid_price;
            
            debug!("MeteoraDlmm prices calculated - Buy: {}, Sell: {}", buy_price, sell_price);
            Some((buy_price, sell_price))
        }
        AnyPoolConfig::MeteoraDammV2(ref c) => {
            debug!("Calculating prices for MeteoraDammV2 pool");
            let sol_to_token = c.data
                .mid_price_for_quick_estimate(sol_mint, &token_mint)
                .await
                .ok()?;
            let token_to_sol = c.data
                .mid_price_for_quick_estimate(&token_mint, sol_mint)
                .await
                .ok()?;
            
            let buy_price = Decimal::ONE / sol_to_token.mid_price;
            let sell_price = token_to_sol.mid_price;
            
            debug!("MeteoraDammV2 prices calculated - Buy: {}, Sell: {}", buy_price, sell_price);
            Some((buy_price, sell_price))
        }
        AnyPoolConfig::Unsupported => {
            debug!("Unsupported pool config type");
            None
        }
    }
}