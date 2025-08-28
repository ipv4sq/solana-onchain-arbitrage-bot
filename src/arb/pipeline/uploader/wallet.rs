use anyhow::Result;
use once_cell::sync::Lazy;
use solana_sdk::signature::{read_keypair_file, Keypair};

static WALLET_FILE_PATH: Lazy<String> = Lazy::new(|| {
    let env = std::env::var("WALLET_FILE_PATH")
        .unwrap_or("/Users/l/Downloads/test_jz.json".to_string())
        .to_lowercase();
    return env;
});

pub fn get_wallet() -> Keypair {
    let wallet_json_path: String = WALLET_FILE_PATH.clone();
    let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
    wallet
}
