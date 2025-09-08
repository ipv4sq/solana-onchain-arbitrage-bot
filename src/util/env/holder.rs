use once_cell::sync::Lazy;
use serde::Deserialize;

pub static ENV_CONFIG: Lazy<EnvironmentStruct> = Lazy::new(|| {
    EnvironmentStruct::load_from_env().expect("Failed to load environment configuration")
});

#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentStruct {
    pub database_url: String,
    pub grpc_url: String,
    pub grpc_token: String,
    pub solana_rpc_url: String,
}

impl EnvironmentStruct {
    fn load_from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            grpc_url: std::env::var("GRPC_URL")?,
            grpc_token: std::env::var("GRPC_TOKEN")?,
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")?,
        })
    }
}
