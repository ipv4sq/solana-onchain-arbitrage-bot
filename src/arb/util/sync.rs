use crate::arb::global::client::rpc::rpc_client;
use crate::arb::util::alias::AResult;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::Duration;
use tokio::{select, sync::mpsc, time};

struct Request {
    address: Pubkey,
    response_tx: mpsc::Sender<AResult<Account>>,
}

async fn the_entry() {
    let (_tx, mut pipeline) = mpsc::channel::<Request>(100);
    async fn async_call() {
        let (_sx, _rx) = mpsc::channel::<Pubkey>(100);
        todo!()
    }
    let mut ticker = time::interval(Duration::from_secs(400));
    let mut current_batch: HashMap<Pubkey, Request> = HashMap::new();

    loop {
        if current_batch.len() >= 100 {
            break;
        }
        select! {
            Some(req) = pipeline.recv() => {
                current_batch.insert(req.address, req);
            }
            _ = ticker.tick() => {
                break
            }
        }
    }

    let public_keys = current_batch.keys().cloned().collect::<Vec<_>>();

    let response = rpc_client().get_multiple_accounts(&public_keys).await;
    match response {
        Ok(accounts) => {
            for (pubkey, account_option) in public_keys.iter().zip(accounts.iter()) {
                if let Some(request) = current_batch.remove(pubkey) {
                    let result = account_option
                        .as_ref()
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Account not found: {}", pubkey));
                    let _ = request.response_tx.send(result).await;
                }
            }
        }
        Err(e) => {
            for (_, request) in current_batch {
                let _ = request
                    .response_tx
                    .send(Err(anyhow::anyhow!("RPC error: {}", e)))
                    .await;
            }
        }
    }
}
