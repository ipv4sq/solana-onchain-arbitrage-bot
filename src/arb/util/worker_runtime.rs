use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

#[allow(non_upper_case_globals)]
pub static ArbitrageOpportunityWorker: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .max_blocking_threads(8)
        .thread_name("arb-worker")
        .enable_all()
        .build()
        .expect("Failed to create worker runtime")
});
