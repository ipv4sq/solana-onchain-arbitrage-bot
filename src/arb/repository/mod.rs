// Transaction management // Central repository manager

// Domain Models
pub mod entity; // SeaORM entity models

// Repository Implementations
pub mod repositories; // Repository implementations

// Core components
pub mod core;

// Optional examples
#[cfg(feature = "examples")]
pub mod usage_example;

pub use core::error::RepositoryResult;
pub use core::manager::{get_repository_manager, RepositoryManager};
pub use core::traits::*;
