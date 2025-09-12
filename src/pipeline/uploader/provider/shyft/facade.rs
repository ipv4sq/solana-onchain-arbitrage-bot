use crate::sdk::rpc::methods::transaction::send_transaction;
use crate::util::alias::AResult;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

pub async fn send_shyft_transaction(tx: &VersionedTransaction) -> AResult<Signature> {
    send_transaction(tx).await
}
