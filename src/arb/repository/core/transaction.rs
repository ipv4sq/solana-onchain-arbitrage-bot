use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::future::Future;

use crate::arb::repository::core::error::{RepositoryError, RepositoryResult};

pub struct TransactionManager<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> TransactionManager<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn execute<F, R, Fut>(&self, f: F) -> RepositoryResult<R>
    where
        F: FnOnce(&DatabaseTransaction) -> Fut,
        Fut: Future<Output = RepositoryResult<R>>,
    {
        let txn = self.conn
            .begin()
            .await
            .map_err(|e| RepositoryError::TransactionFailed {
                message: format!("Failed to begin transaction: {}", e),
            })?;

        match f(&txn).await {
            Ok(result) => {
                txn.commit()
                    .await
                    .map_err(|e| RepositoryError::TransactionFailed {
                        message: format!("Failed to commit transaction: {}", e),
                    })?;
                Ok(result)
            }
            Err(e) => {
                txn.rollback()
                    .await
                    .map_err(|e| RepositoryError::TransactionFailed {
                        message: format!("Failed to rollback transaction: {}", e),
                    })?;
                Err(e)
            }
        }
    }

    pub async fn execute_with_retry<F, R, Fut>(
        &self,
        f: F,
        max_retries: u32,
    ) -> RepositoryResult<R>
    where
        F: Fn(&DatabaseTransaction) -> Fut,
        Fut: Future<Output = RepositoryResult<R>>,
    {
        let mut attempts = 0;
        
        loop {
            match self.execute(&f).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts < max_retries => {
                    attempts += 1;
                    tracing::warn!(
                        "Transaction failed (attempt {}/{}): {:?}",
                        attempts,
                        max_retries,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempts as u64)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}