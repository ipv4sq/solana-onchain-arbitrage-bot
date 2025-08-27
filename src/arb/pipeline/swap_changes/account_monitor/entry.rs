use crate::arb::database::repositories::pool_repo::PoolRecordRepository;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::global::trace::types::Trace;
use crate::arb::pipeline::swap_changes::account_monitor::pool_update::PoolUpdate;
use crate::arb::pipeline::trade_strategy::entry::on_pool_update;
use crate::arb::util::structs::lazy_cache::LazyCache;
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use tracing::{debug, info, warn};

#[allow(non_upper_case_globals)]
pub static NonPoolBlocklist: LazyCache<Pubkey, bool> = LazyCache::new();

pub async fn process_pool_update(update: PoolUpdate, trace: Trace) -> Result<()> {
    let pool_addr = update.pool();

    if !determine_if_pool_address(pool_addr).await {
        return Ok(());
    }

    if update.is_initial() {
        return Ok(());
    }

    if !update.data_changed() {
        return Ok(());
    }

    info!("Pool data changed for: {}", pool_addr);
    on_pool_update(update, trace).await;

    Ok(())
}

async fn determine_if_pool_address(addr: &Pubkey) -> bool {
    if NonPoolBlocklist.contains_key(addr) {
        return false;
    }

    if let Ok(account) = rpc_client().get_account(addr).await {
        let data_len = account.data.len();
        if data_len < 500 || data_len > 2000 {
            NonPoolBlocklist.put(*addr, true);
            return false;
        }
    } else {
        NonPoolBlocklist.put(*addr, true);
        return false;
    }

    let pool = match PoolRecordRepository::ensure_exists(addr).await {
        Some(p) => p,
        None => {
            warn!("Cannot find pool {} in database, adding to blocklist", addr);
            NonPoolBlocklist.put(*addr, true);
            return false;
        }
    };

    if pool.base_mint.0 != Mints::WSOL && pool.quote_mint.0 != Mints::WSOL {
        NonPoolBlocklist.put(*addr, true);
        return false;
    }

    true
}
