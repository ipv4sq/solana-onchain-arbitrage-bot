use crate::lined_err;
use crate::sdk::rpc::client::rpc_client;
use crate::sdk::rpc::methods::limiter::QueryRateLimiter;
use crate::util::alias::AResult;
use crate::util::traits::option::OptionExt;
use anyhow::anyhow;
use mpsc::{channel, Receiver};
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, OnceCell};
use tokio::{select, time};
use tracing::info;

struct Request {
    address: Pubkey,
    on_response: Sender<AResult<Account>>,
}

static CHANNEL: OnceCell<Sender<Request>> = OnceCell::const_new();

async fn get_sender() -> &'static Sender<Request> {
    CHANNEL
        .get_or_init(|| async {
            let (tx, rx) = channel::<Request>(1000);

            // Spawn a dedicated thread with its own runtime for the buffered get account loop
            thread::spawn(move || {
                let runtime = Runtime::new()
                    .expect("Failed to create dedicated runtime for buffered_get_account");
                runtime.block_on(async {
                    info!("Started dedicated buffered_get_account worker thread");
                    loop_forever(rx).await;
                });
            });

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
    rx.recv()
        .await
        .or_else_err(lined_err!("channel closed unexpectedly"))?
}

pub async fn buffered_get_account_batch(addresses: &[Pubkey]) -> AResult<Vec<Option<Account>>> {
    let mut receivers = Vec::with_capacity(addresses.len());
    let sender = get_sender().await;

    for address in addresses {
        let (tx, rx) = channel::<AResult<Account>>(1);
        let request = Request {
            address: *address,
            on_response: tx,
        };
        sender.send(request).await?;
        receivers.push(rx);
    }

    let mut results = Vec::with_capacity(addresses.len());
    for mut rx in receivers {
        let result = rx.recv().await.or_err("channel closed unexpectedly")?;
        results.push(result.ok());
    }

    Ok(results)
}

pub async fn buffered_get_account_batch_map(
    addresses: &[Pubkey],
) -> AResult<HashMap<Pubkey, Account>> {
    let batch_results = buffered_get_account_batch(addresses).await?;

    let mut map = HashMap::with_capacity(addresses.len());
    for (address, account_opt) in addresses.iter().zip(batch_results.into_iter()) {
        if let Some(account) = account_opt {
            map.insert(*address, account);
        }
    }

    Ok(map)
}

async fn loop_forever(mut pipeline: Receiver<Request>) {
    loop {
        every_batch(&mut pipeline).await;
    }
}

async fn every_batch(pipeline: &mut Receiver<Request>) {
    let mut ticker = time::interval(Duration::from_millis(400));
    let mut current_batch: HashMap<Pubkey, Vec<Request>> = HashMap::new();

    loop {
        if current_batch.len() >= 100 {
            break;
        }
        select! {
            Some(req) = pipeline.recv() => {
                current_batch.entry(req.address).or_insert_with(Vec::new).push(req);
            }
            _ = ticker.tick() => {
                break
            }
        }
    }

    if current_batch.len() == 0 {
        return;
    }

    if let Err(rate_limit_err) = QueryRateLimiter.try_acquire_err() {
        for (_, requests) in current_batch {
            for request in requests {
                let _ = request
                    .on_response
                    .send(Err(rate_limit_err.clone().into()))
                    .await;
            }
        }
        return;
    }

    let public_keys = current_batch.keys().cloned().collect::<Vec<_>>();
    let response = rpc_client().get_multiple_accounts(&public_keys).await;

    match response {
        Ok(accounts) => {
            for (pubkey, account_option) in public_keys.iter().zip(accounts.iter()) {
                if let Some(requests) = current_batch.remove(pubkey) {
                    for request in requests {
                        let result = match account_option {
                            Some(account) => Ok(account.clone()),
                            None => Err(anyhow!("{} Not found", pubkey)),
                        };

                        if let Err(e) = request.on_response.send(result).await {
                            info!("on_response dropped before send: {} ({})", e, pubkey)
                        }
                    }
                }
            }
        }
        Err(e) => {
            for (_, requests) in current_batch {
                for request in requests {
                    let _ = request
                        .on_response
                        .send(Err(anyhow::anyhow!("RPC error: {}", e)))
                        .await;
                }
            }
        }
    }
}
