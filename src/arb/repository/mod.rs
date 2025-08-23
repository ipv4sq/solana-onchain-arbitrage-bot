// Transaction management // Central repository manager

// Domain Models
pub mod entity; // SeaORM entity models

// Repository Implementations
pub mod repositories; // Repository implementations

// Legacy/Compatibility
pub mod pool_repository; // Keep for backward compatibility

// Optional examples
pub mod core;
#[cfg(feature = "examples")]
pub mod usage_example;

pub use core::error::RepositoryResult;
pub use core::manager::{get_repository_manager, RepositoryManager};
pub use core::traits::*;
