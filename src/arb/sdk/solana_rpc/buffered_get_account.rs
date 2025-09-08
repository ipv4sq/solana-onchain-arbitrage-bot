use crate::arb::sdk::solana_rpc::rpc::rpc_client;
use crate::arb::util::alias::AResult;
use crate::arb::util::traits::option::OptionExt;
use anyhow::anyhow;
use mpsc::{channel, Receiver};
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, OnceCell};
use tokio::{select, time};

struct Request {
    address: Pubkey,
    on_response: Sender<AResult<Account>>,
}

static CHANNEL: OnceCell<Sender<Request>> = OnceCell::const_new();

async fn get_sender() -> &'static Sender<Request> {
    CHANNEL
        .get_or_init(|| async {
            let (tx, rx) = channel::<Request>(1000);
            tokio::spawn(loop_forever(rx));
            tx
        })
        .await
}

pub async fn buffered_get_account(address: &Pubkey) -> AResult<Account> {
    let (tx, mut rx) = channel::<AResult<Account>>(1);
    let request = Request {
        address: *address,
        on_response: tx,
    };
    let _ = get_sender().await.send(request).await?;
    rx.recv().await.or_err("channel closed unexpectedly")?
}

async fn loop_forever(mut pipeline: Receiver<Request>) {
    loop {
        every_batch(&mut pipeline).await;
    }
}

async fn every_batch(pipeline: &mut Receiver<Request>) {
    let mut ticker = time::interval(Duration::from_millis(400));
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

    if current_batch.len() == 0 {
        return;
    }

    let public_keys = current_batch.keys().cloned().collect::<Vec<_>>();
    let response = rpc_client().get_multiple_accounts(&public_keys).await;

    match response {
        Ok(accounts) => {
            let some_accounts = accounts
                .iter()
                .filter_map(|a| a.as_ref())
                .collect::<Vec<_>>();

            for account in some_accounts {
                if let Some(x) = current_batch.remove(&account.owner) {
                    let _ = x.on_response.send(Ok(account.clone())).await;
                }
            }

            for (leftover, request) in current_batch {
                let _ = request
                    .on_response
                    .send(Err(anyhow!("{} Not found", leftover)))
                    .await;
            }
        }
        Err(e) => {
            for (_, request) in current_batch {
                let _ = request
                    .on_response
                    .send(Err(anyhow::anyhow!("RPC error: {}", e)))
                    .await;
            }
        }
    }
}
