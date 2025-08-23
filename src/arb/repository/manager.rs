use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::arb::repository::{
    database::DatabaseManager,
    repositories::*,
    transaction::TransactionManager,
    error::RepositoryResult,
};

/// Central repository manager that provides access to all repositories
pub struct RepositoryManager {
    db: Arc<DatabaseConnection>,
}

impl RepositoryManager {
    /// Create a new repository manager from a database manager
    pub async fn new() -> RepositoryResult<Self> {
        let manager = DatabaseManager::new().await
            .map_err(|e| crate::arb::repository::error::RepositoryError::Connection(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(manager.connection().clone()),
        })
    }

    /// Create from an existing database connection
    pub fn from_connection(db: DatabaseConnection) -> Self {
        Self {
            db: Arc::new(db),
        }
    }

    /// Get the database connection
    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Get a pool repository
    pub fn pools(&self) -> PoolRepository {
        PoolRepository::new(&self.db)
    }

    /// Get a swap repository
    pub fn swaps(&self) -> SwapRepository {
        SwapRepository::new(&self.db)
    }

    /// Get an arbitrage repository
    pub fn arbitrage(&self) -> ArbitrageRepository {
        ArbitrageRepository::new(&self.db)
    }

    /// Get a metrics repository
    pub fn metrics(&self) -> MetricsRepository {
        MetricsRepository::new(&self.db)
    }

    /// Get a transaction manager for database transactions
    pub fn transaction_manager(&self) -> TransactionManager {
        TransactionManager::new(&self.db)
    }

    /// Execute a function within a database transaction
    pub async fn with_transaction<F, R, Fut>(&self, f: F) -> RepositoryResult<R>
    where
        F: FnOnce(&sea_orm::DatabaseTransaction) -> Fut,
        Fut: std::future::Future<Output = RepositoryResult<R>>,
    {
        self.transaction_manager().execute(f).await
    }

    /// Check database health
    pub async fn health_check(&self) -> RepositoryResult<bool> {
        use sea_orm::ConnectionTrait;
        
        self.db
            .execute_unprepared("SELECT 1")
            .await
            .map(|_| true)
            .map_err(|e| crate::arb::repository::error::RepositoryError::Connection(e.to_string()))
    }
}

// Singleton instance for global access
use tokio::sync::OnceCell;
static REPOSITORY_MANAGER: OnceCell<Arc<RepositoryManager>> = OnceCell::const_new();

/// Get or initialize the global repository manager
pub async fn get_repository_manager() -> RepositoryResult<Arc<RepositoryManager>> {
    REPOSITORY_MANAGER
        .get_or_init(|| async {
            Arc::new(RepositoryManager::new().await.expect("Failed to initialize repository manager"))
        })
        .await
        .clone()
        .try_into()
        .map_err(|_| crate::arb::repository::error::RepositoryError::Connection(
            "Failed to get repository manager instance".to_string()
        ))
}