use super::rpc::rpc_client;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_client::rpc_response::{Response, RpcSimulateTransactionResult};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

pub async fn get_account(address: &Pubkey) -> Result<Account, solana_client::client_error::ClientError> {
    rpc_client().get_account(address).await
}

pub async fn get_account_with_commitment(
    address: &Pubkey,
    commitment: CommitmentConfig,
) -> Result<Response<Option<Account>>, solana_client::client_error::ClientError> {
    rpc_client().get_account_with_commitment(address, commitment).await
}

pub async fn get_account_data(address: &Pubkey) -> Result<Vec<u8>, solana_client::client_error::ClientError> {
    rpc_client().get_account_data(address).await
}

pub async fn get_multiple_accounts(
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>, solana_client::client_error::ClientError> {
    rpc_client().get_multiple_accounts(pubkeys).await
}

pub async fn get_multiple_accounts_with_commitment(
    pubkeys: &[Pubkey],
    commitment: CommitmentConfig,
) -> Result<Response<Vec<Option<Account>>>, solana_client::client_error::ClientError> {
    rpc_client().get_multiple_accounts_with_commitment(pubkeys, commitment).await
}

pub async fn get_latest_blockhash() -> Result<Hash, solana_client::client_error::ClientError> {
    rpc_client().get_latest_blockhash().await
}

pub async fn get_latest_blockhash_with_commitment(
    commitment: CommitmentConfig,
) -> Result<(Hash, u64), solana_client::client_error::ClientError> {
    rpc_client().get_latest_blockhash_with_commitment(commitment).await
}

pub async fn send_transaction(
    transaction: &VersionedTransaction,
) -> Result<Signature, solana_client::client_error::ClientError> {
    rpc_client().send_transaction(transaction).await
}

pub async fn send_transaction_with_config(
    transaction: &VersionedTransaction,
    config: RpcSendTransactionConfig,
) -> Result<Signature, solana_client::client_error::ClientError> {
    rpc_client().send_transaction_with_config(transaction, config).await
}

pub async fn simulate_transaction(
    transaction: &VersionedTransaction,
) -> Result<Response<RpcSimulateTransactionResult>, solana_client::client_error::ClientError> {
    rpc_client().simulate_transaction(transaction).await
}

pub async fn simulate_transaction_with_config(
    transaction: &VersionedTransaction,
    config: RpcSimulateTransactionConfig,
) -> Result<Response<RpcSimulateTransactionResult>, solana_client::client_error::ClientError> {
    rpc_client().simulate_transaction_with_config(transaction, config).await
}

pub async fn get_transaction_with_config(
    signature: &Signature,
    config: solana_client::rpc_config::RpcTransactionConfig,
) -> Result<EncodedConfirmedTransactionWithStatusMeta, solana_client::client_error::ClientError> {
    rpc_client().get_transaction_with_config(signature, config).await
}