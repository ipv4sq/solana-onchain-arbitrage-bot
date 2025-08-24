pub mod mint_do;
pub mod pool_do;

// mint
pub use mint_do::Entity as MintRecordTable;
pub use mint_do::Model as MintRecord;

// pool record
pub use pool_do::Entity as PoolRecordTable;
pub use pool_do::Model as PoolRecord;
