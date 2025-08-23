use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;
use std::time::Duration;

pub struct DatabaseManager {
    connection: DatabaseConnection,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self> {
        dotenv::dotenv().ok();
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

        let mut opt = ConnectOptions::new(database_url.clone());
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8));

        let connection = Database::connect(opt).await?;

        Ok(Self { connection })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
