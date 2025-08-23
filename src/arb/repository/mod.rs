pub mod entity;
pub mod database;
pub mod error;
pub mod traits;
pub mod transaction;
pub mod repositories;
pub mod manager;
pub mod pool_repository; // Keep for backward compatibility
mod tables;

use anyhow::Result;

// Re-export main components
pub use database::DatabaseManager;
pub use error::{RepositoryError, RepositoryResult};
pub use manager::{RepositoryManager, get_repository_manager};
pub use repositories::*;
pub use traits::*;
pub use transaction::TransactionManager;

// Keep backward compatibility
pub use pool_repository::PoolRepository;
