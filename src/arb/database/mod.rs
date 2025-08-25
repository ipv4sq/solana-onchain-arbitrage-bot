// Domain Models
pub mod entity; // SeaORM entity models

// Custom types for SeaORM
pub mod columns;

// Repository Implementations
pub mod repositories; // Repository implementations

// Core components
pub mod core;

pub use core::error::RepositoryResult;
pub use core::traits::*;
