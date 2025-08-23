use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Entity not found: {entity}")]
    NotFound { entity: String },
    
    #[error("Duplicate entry: {entity}")]
    DuplicateEntry { entity: String },
    
    #[error("Invalid data: {message}")]
    InvalidData { message: String },
    
    #[error("Transaction failed: {message}")]
    TransactionFailed { message: String },
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;