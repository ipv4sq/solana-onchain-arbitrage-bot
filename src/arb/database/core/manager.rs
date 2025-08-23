use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::arb::database::repositories::*;

/// Central repository manager that provides access to all repositories
pub struct RepositoryManager {
    db: Arc<DatabaseConnection>,
}

impl RepositoryManager {
    /// Create a new repository manager from a database manager
    pub async fn new() -> RepositoryResult<Self> {
        let manager = DatabaseManager::new().await.map_err(|e| {
            crate::arb::database::core::error::RepositoryError::Connection(e.to_string())
        })?;

        Ok(Self {
            db: Arc::new(manager.connection().clone()),
        })
    }

    /// Create from an existing database connection
    pub fn from_connection(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn transaction_manager(&self) -> TransactionManager {
        TransactionManager::new(&self.db)
    }

    pub async fn with_transaction<F, R, Fut>(&self, f: F) -> RepositoryResult<R>
    where
        F: FnOnce(&sea_orm::DatabaseTransaction) -> Fut,
        Fut: std::future::Future<Output = RepositoryResult<R>>,
    {
        self.transaction_manager().execute(f).await
    }

    pub async fn health_check(&self) -> RepositoryResult<bool> {
        use sea_orm::{ConnectionTrait, Statement};

        self.db
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT 1".to_string(),
            ))
            .await
            .map(|_| true)
            .map_err(|e| {
                crate::arb::database::core::error::RepositoryError::Connection(e.to_string())
            })
    }
}

// Singleton instance for global access
use crate::arb::database::core::database::DatabaseManager;
use crate::arb::database::core::error::RepositoryResult;
use crate::arb::database::core::transaction::TransactionManager;
use tokio::sync::OnceCell;

static REPOSITORY_MANAGER: OnceCell<Arc<RepositoryManager>> = OnceCell::const_new();

pub async fn get_repository_manager() -> RepositoryResult<Arc<RepositoryManager>> {
    REPOSITORY_MANAGER
        .get_or_init(|| async {
            Arc::new(
                RepositoryManager::new()
                    .await
                    .expect("Failed to initialize repository manager"),
            )
        })
        .await
        .clone()
        .try_into()
        .map_err(|_| {
            crate::arb::database::core::error::RepositoryError::Connection(
                "Failed to get repository manager instance".to_string(),
            )
        })
}
