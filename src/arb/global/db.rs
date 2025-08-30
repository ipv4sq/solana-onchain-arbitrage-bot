use anyhow::Result;
use once_cell::sync::Lazy;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;
use std::time::Duration;

static DB_CONNECTION: Lazy<DatabaseConnection> = Lazy::new(|| {
    // This initialization happens in a blocking context
    // For tests, we need to ensure they call init_db_async first
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            create_connection().await.expect("Failed to initialize database")
        })
    })
    .join()
    .unwrap()
});

async fn create_connection() -> Result<DatabaseConnection> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(2))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(false);

    Ok(Database::connect(opt).await?)
}

pub async fn init_db() -> Result<()> {
    // Force lazy initialization
    let _ = &*DB_CONNECTION;
    Ok(())
}

pub fn get_db() -> &'static DatabaseConnection {
    &*DB_CONNECTION
}

pub fn is_db_initialized() -> bool {
    // With Lazy, it's always initialized when accessed
    true
}

#[cfg(test)]
pub async fn ensure_test_db() -> Result<()> {
    if !is_db_initialized() {
        init_db().await?;
    }
    Ok(())
}
