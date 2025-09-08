use crate::arb::global::client::rpc::rpc_client;
use crate::arb::global::constant::mint::Mints;
use maplit::hashset;
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::{select, sync::mpsc, time};

struct Request {
    address: Pubkey,
    response_rx: mpsc::Receiver<String>,
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
    let a = rpc_client().get_account(&Mints::WSOL).await.unwrap();
    // get multiaccount
}
