use crate::convention::chain::mapper::traits::ToUnified;
use crate::convention::chain::Transaction;
use crate::lined_err;
use crate::sdk::rpc::client;
use crate::sdk::rpc::client::rpc_client;
use crate::util::alias::AResult;
use crate::util::traits::signature::ToSignature;
use solana_program::address_lookup_table::AddressLookupTableAccount;
use solana_program::hash::Hash;
use solana_program::instruction::Instruction;
use solana_program::message::v0::Message;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::VersionedTransaction;
use tracing::info;

pub async fn send_transaction(tx: &VersionedTransaction) -> AResult<Signature> {
    rpc_client().send_transaction(tx).await.map_err(Into::into)
}

pub async fn send_tx_with_retry(
    tx: &VersionedTransaction,
    max_retries: u64,
) -> anyhow::Result<Signature> {
    let tx_bytes = bincode::serialize(tx)?;
    let tx_size = tx_bytes.len();
    info!("Transaction size after compilation: {} bytes", tx_size);

    client::rpc_client()
        .send_transaction_with_config(
            tx,
            solana_client::rpc_config::RpcSendTransactionConfig {
                skip_preflight: true,
                max_retries: Some(max_retries as usize),
                preflight_commitment: Some(CommitmentLevel::Processed),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| lined_err!("Failed to send transaction: {}", e))
}

pub async fn fetch_tx(signature: &str) -> AResult<Transaction> {
    client::rpc_client()
        .get_transaction_with_config(&signature.to_sig(), client::json_config())
        .await
        .map_err(|e| lined_err!("Failed to fetch transaction: {}", e))?
        .to_unified()
}

pub fn compile_instruction_to_tx(
    wallet: &Keypair,
    instructions: Vec<Instruction>,
    alts: &[AddressLookupTableAccount],
    blockhash: Hash,
) -> AResult<VersionedTransaction> {
    let message = Message::try_compile(&wallet.pubkey(), &instructions, alts, blockhash)?;
    let tx = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[wallet],
    )?;
    Ok(tx)
}
