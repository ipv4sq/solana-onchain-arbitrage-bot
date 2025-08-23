use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;
use std::sync::OnceLock;
use std::time::Duration;

static DB_CONNECTION: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init_db() -> Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false);

    let connection = Database::connect(opt).await?;
    
    DB_CONNECTION
        .set(connection)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;
    
    Ok(())
}

pub fn get_db() -> &'static DatabaseConnection {
    DB_CONNECTION
        .get()
        .expect("Database not initialized. Call init_db() first")
}

pub fn is_db_initialized() -> bool {
    DB_CONNECTION.get().is_some()
}

#[cfg(test)]
mod test_init {
    use super::*;
    use ctor::ctor;
    
    #[ctor]
    fn init_test_db() {
        // Initialize tokio runtime for test database setup
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        // Only initialize if not already done
        if !is_db_initialized() {
            runtime.block_on(async {
                if let Err(e) = init_db().await {
                    eprintln!("Warning: Failed to initialize test database: {}", e);
                    eprintln!("Database-dependent tests may fail");
                }
            });
        }
    }
}