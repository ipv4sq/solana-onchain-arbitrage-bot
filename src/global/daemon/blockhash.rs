use crate::sdk::solana_rpc::methods::block::get_latest_blockhash;
use anyhow::Result;
use parking_lot::RwLock;
use solana_sdk::hash::Hash;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::OnceCell;
use tokio::time::interval;

struct BlockhashHolder {
    blockhash: RwLock<Hash>,
}

impl BlockhashHolder {
    fn new() -> Self {
        Self {
            blockhash: RwLock::new(Hash::default()),
        }
    }

    fn get(&self) -> Hash {
        *self.blockhash.read()
    }

    fn start_updater(self: &Arc<Self>) {
        let holder = self.clone();

        // Spawn a dedicated OS thread for blockhash updates
        thread::Builder::new()
            .name("blockhash-updater".to_string())
            .spawn(move || {
                tracing::info!("Blockhash updater thread started");

                // Create a dedicated single-threaded runtime for this thread
                let runtime = Builder::new_current_thread()
                    .enable_all()
                    .thread_name("blockhash-runtime")
                    .build()
                    .expect("Failed to create runtime for blockhash updater");

                // Run the update loop forever on this dedicated thread
                runtime.block_on(async move {
                    let mut interval = interval(Duration::from_millis(400));

                    loop {
                        interval.tick().await;

                        match get_latest_blockhash().await {
                            Ok(new_blockhash) => {
                                *holder.blockhash.write() = new_blockhash;
                                tracing::trace!("Blockhash updated: {}", new_blockhash);
                            }
                            Err(e) => {
                                tracing::error!("Failed to fetch blockhash: {:?}", e);
                            }
                        }
                    }
                });
            })
            .expect("Failed to spawn blockhash updater thread");
    }
}

static GLOBAL_BLOCKHASH: OnceCell<Arc<BlockhashHolder>> = OnceCell::const_new();

async fn ensure_initialized() -> Result<()> {
    GLOBAL_BLOCKHASH
        .get_or_init(|| async {
            let holder = Arc::new(BlockhashHolder::new());

            // Fetch initial blockhash
            if let Ok(initial_hash) = get_latest_blockhash().await {
                *holder.blockhash.write() = initial_hash;
            }

            holder.start_updater();
            holder
        })
        .await;

    Ok(())
}

pub async fn initialize() -> Result<()> {
    ensure_initialized().await
}

pub async fn get_blockhash() -> Result<Hash> {
    ensure_initialized().await?;
    Ok(GLOBAL_BLOCKHASH.get().unwrap().get())
}
