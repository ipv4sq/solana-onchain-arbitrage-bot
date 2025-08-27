use crate::arb::database::repositories::MintRecordRepository;
use crate::arb::util::alias::MintAddress;
use dashmap::DashMap;
use rust_decimal::Decimal;
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PoolPrice {
    pub pool_address: Pubkey,
    pub price_in_sol: Decimal,
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub buy_pool: Pubkey,
    pub sell_pool: Pubkey,
    pub token_mint: MintAddress,
    pub buy_price: Decimal,
    pub sell_price: Decimal,
    pub spread: Decimal,
}

pub struct MintPriceTracker {
    mint: MintAddress,
    buy_prices: RwLock<BTreeMap<String, PoolPrice>>,
    sell_prices: RwLock<BTreeMap<String, PoolPrice>>,
}

impl MintPriceTracker {
    pub fn new(mint: MintAddress) -> Self {
        Self {
            mint,
            buy_prices: RwLock::new(BTreeMap::new()),
            sell_prices: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn update_prices(&self, pool_address: Pubkey, buy_price: Decimal, sell_price: Decimal) {
        {
            let buy_key = format!("{:.18}_{}", buy_price, pool_address);
            let mut buy_prices = self.buy_prices.write().unwrap();
            buy_prices.insert(
                buy_key,
                PoolPrice {
                    pool_address,
                    price_in_sol: buy_price,
                },
            );
        }

        {
            let sell_key = format!("{:.18}_{}", sell_price, pool_address);
            let mut sell_prices = self.sell_prices.write().unwrap();
            sell_prices.insert(
                sell_key,
                PoolPrice {
                    pool_address,
                    price_in_sol: sell_price,
                },
            );
        }
    }

    pub fn detect_arbitrage(&self) -> Option<ArbitrageOpportunity> {
        // Minimum profit threshold: 1.5% (adjust as needed for gas costs and slippage)
        const MIN_PROFIT_PERCENTAGE: Decimal = Decimal::from_parts(15, 0, 0, false, 1); // 1.5%

        let buy_prices = self.buy_prices.read().unwrap();
        let sell_prices = self.sell_prices.read().unwrap();
        let mint_symbol = MintRecordRepository::get_symbol_from_cache_sync(&self.mint);
        if buy_prices.is_empty() || sell_prices.is_empty() {
            return None;
        }

        let min_buy = buy_prices.iter().next()?;
        let max_sell = sell_prices.iter().last()?;
        
        // Skip if buy and sell are the same pool
        if min_buy.1.pool_address == max_sell.1.pool_address {
            return None;
        }

        if max_sell.1.price_in_sol > min_buy.1.price_in_sol {
            let sol_in = Decimal::from(1);
            let tokens_bought = sol_in / min_buy.1.price_in_sol;
            let sol_out = tokens_bought * max_sell.1.price_in_sol;
            let profit = sol_out - sol_in;
            let profit_percentage = profit * Decimal::from(100);

            if profit_percentage >= MIN_PROFIT_PERCENTAGE {
                tracing::info!(
                    "🎯 Arbitrage detected for {}: 1 SOL → {:.6} {} → {:.6} SOL (profit: {:.6} SOL, {:.2}%)",
                    self.mint,
                    tokens_bought,
                    mint_symbol,
                    sol_out,
                    profit,
                    profit_percentage
                );

                Some(ArbitrageOpportunity {
                    buy_pool: min_buy.1.pool_address,
                    sell_pool: max_sell.1.pool_address,
                    token_mint: self.mint,
                    buy_price: min_buy.1.price_in_sol,
                    sell_price: max_sell.1.price_in_sol,
                    spread: max_sell.1.price_in_sol - min_buy.1.price_in_sol,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn clear(&self) {
        self.buy_prices.write().unwrap().clear();
        self.sell_prices.write().unwrap().clear();
    }

    pub fn get_best_buy_price(&self) -> Option<PoolPrice> {
        let prices = self.buy_prices.read().unwrap();
        prices.values().next().cloned()
    }

    pub fn get_best_sell_price(&self) -> Option<PoolPrice> {
        let prices = self.sell_prices.read().unwrap();
        prices.values().last().cloned()
    }
}

lazy_static::lazy_static! {
    static ref PRICE_TRACKERS: DashMap<MintAddress, Arc<MintPriceTracker>> = DashMap::new();
}

pub fn update_pool_prices(
    token_mint: MintAddress,
    pool_address: Pubkey,
    buy_price: Decimal,
    sell_price: Decimal,
) {
    let tracker = PRICE_TRACKERS
        .entry(token_mint)
        .or_insert_with(|| Arc::new(MintPriceTracker::new(token_mint)));
    tracker.update_prices(pool_address, buy_price, sell_price);
}

pub fn detect_arbitrage(token_mint: MintAddress) -> Option<ArbitrageOpportunity> {
    PRICE_TRACKERS.get(&token_mint)?.detect_arbitrage()
}

pub fn clear_prices_for_token(token_mint: MintAddress) {
    if let Some(tracker) = PRICE_TRACKERS.get(&token_mint) {
        tracker.clear();
    }
}
