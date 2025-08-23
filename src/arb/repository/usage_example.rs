use crate::arb::global::enums::dex_type::DexType;
use crate::arb::repository::{get_repository_manager, RepositoryManager, RepositoryResult};
use solana_sdk::pubkey::Pubkey;

/// Example of using the repository pattern with dependency injection
pub struct ArbitrageService {
    repo_manager: std::sync::Arc<RepositoryManager>,
}

impl ArbitrageService {
    pub async fn new() -> RepositoryResult<Self> {
        let repo_manager = get_repository_manager().await?;
        Ok(Self { repo_manager })
    }

    /// Example: Record a new pool and its metrics
    pub async fn record_new_pool(
        &self,
        pool_id: &Pubkey,
        desired_mint: &Pubkey,
        other_mint: &Pubkey,
        dex_type: DexType,
    ) -> RepositoryResult<()> {
        // Use transaction to ensure atomicity
        self.repo_manager
            .with_transaction(|txn| {
                Box::pin(async move {
                    // 1. Create or update pool
                    let pool_repo = crate::arb::repository::repositories::PoolRepository::new(txn);
                    pool_repo
                        .upsert(
                            pool_id.to_string(),
                            desired_mint.to_string(),
                            other_mint.to_string(),
                            dex_type,
                        )
                        .await?;

                    Ok(())
                })
            })
            .await
    }

    /// Example: Find arbitrage opportunities
    pub async fn find_arbitrage_opportunities(
        &self,
        mint: &Pubkey,
    ) -> RepositoryResult<Vec<ArbitrageOpportunity>> {
        let pool_repo = self.repo_manager.pools();

        // Find all pools containing this mint
        let pools = pool_repo
            .find_by_mints(&mint.to_string(), &mint.to_string())
            .await?;

        // Get metrics for these pools
        let metrics_repo = self.repo_manager.metrics();

        let mut opportunities = Vec::new();
        for pool in pools {
            if let Ok(metrics) = metrics_repo.find_by_dex_type(&pool.dex_type).await {
                for metric in metrics {
                    opportunities.push(ArbitrageOpportunity {
                        pool_id: pool.pool_id.clone(),
                        dex_type: pool.dex_type,
                        tvl: metric.tvl_usd,
                        volume_24h: metric.volume_24h_usd,
                    });
                }
            }
        }

        Ok(opportunities)
    }

    /// Example: Get analytics dashboard data
    pub async fn get_dashboard_data(&self) -> RepositoryResult<DashboardData> {
        // Parallel queries using different repositories
        let (top_pools, recent_swaps, profitable_arbs, dex_summary) = tokio::join!(
            self.repo_manager.metrics().find_top_by_volume(10),
            self.repo_manager.swaps().find_recent(24),
            self.repo_manager.arbitrage().find_profitable(1000, 10),
            self.repo_manager.metrics().get_dex_summary(),
        );

        Ok(DashboardData {
            top_pools: top_pools?,
            recent_swap_count: recent_swaps?.len(),
            profitable_arbitrages: profitable_arbs?,
            dex_summaries: dex_summary?,
        })
    }

    /// Example: Batch operations
    pub async fn batch_import_pools(
        &self,
        pools: Vec<(String, String, String, DexType)>,
    ) -> RepositoryResult<()> {
        use crate::arb::repository::core::traits::BatchOperations;
        use crate::arb::repository::entity::pool_mints;
        use chrono::Utc;
        use sea_orm::ActiveValue::Set;

        let models: Vec<pool_mints::ActiveModel> = pools
            .into_iter()
            .map(
                |(pool_id, desired_mint, other_mint, dex_type)| pool_mints::ActiveModel {
                    pool_id: Set(pool_id),
                    desired_mint: Set(desired_mint),
                    the_other_mint: Set(other_mint),
                    dex_type: Set(dex_type),
                    created_at: Set(Some(Utc::now())),
                    updated_at: Set(Some(Utc::now())),
                    ..Default::default()
                },
            )
            .collect();

        self.repo_manager.pools().batch_create(models).await
    }

    /// Example: Complex query with pagination
    pub async fn search_pools_paginated(
        &self,
        search_term: &str,
        page: u64,
        per_page: u64,
    ) -> RepositoryResult<PaginatedResult> {
        use crate::arb::repository::core::traits::{Paginate, Search};

        let pool_repo = self.repo_manager.pools();

        // Search and paginate
        let search_results = pool_repo.search(search_term).await?;
        let (items, total_pages) = pool_repo.paginate(page, per_page).await?;

        Ok(PaginatedResult {
            items,
            total_pages,
            current_page: page,
            total_items: search_results.len(),
        })
    }
}

// Data structures for examples
#[derive(Debug)]
pub struct ArbitrageOpportunity {
    pub pool_id: String,
    pub dex_type: DexType,
    pub tvl: rust_decimal::Decimal,
    pub volume_24h: rust_decimal::Decimal,
}

#[derive(Debug)]
pub struct DashboardData {
    pub top_pools: Vec<crate::arb::repository::entity::pool_metrics::Model>,
    pub recent_swap_count: usize,
    pub profitable_arbitrages: Vec<crate::arb::repository::entity::arbitrage_results::Model>,
    pub dex_summaries: Vec<crate::arb::repository::repositories::metrics_repository::DexSummary>,
}

#[derive(Debug)]
pub struct PaginatedResult {
    pub items: Vec<crate::arb::repository::entity::pool_mints::Model>,
    pub total_pages: u64,
    pub current_page: u64,
    pub total_items: usize,
}
