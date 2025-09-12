use crate::util::env::env_config::ENV_CONFIG;
use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tokio::sync::OnceCell;

static DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

async fn create_connection() -> Result<DatabaseConnection> {
    dotenv::dotenv().ok();
    let database_url = ENV_CONFIG.database_url.clone();

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(50)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(3600))
        .sqlx_logging(false);

    Ok(Database::connect(opt).await?)
}

pub async fn init_db() -> Result<()> {
    DB_CONNECTION
        .get_or_try_init(|| async { create_connection().await })
        .await?;
    Ok(())
}

pub async fn must_init_db() {
    init_db().await.unwrap();
}

pub async fn get_db() -> &'static DatabaseConnection {
    DB_CONNECTION
        .get_or_try_init(|| async { create_connection().await })
        .await
        .ok()
        .unwrap()
}
