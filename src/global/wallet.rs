use crate::util::env::env_config::ENV_CONFIG;
use solana_sdk::signature::{read_keypair_file, Keypair};

pub fn get_wallet() -> Keypair {
    let wallet_json_path: String = ENV_CONFIG.wallet_file_path.clone();
    let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
    wallet
}
