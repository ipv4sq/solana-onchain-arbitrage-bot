use crate::sdk::rpc::client::rpc_client;
use solana_program::hash::Hash;

pub async fn get_latest_blockhash() -> Result<Hash, solana_client::client_error::ClientError> {
    rpc_client().get_latest_blockhash().await
}
