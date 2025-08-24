use anyhow::Result;
use solana_sdk::signature::{read_keypair_file, Keypair};

fn get_wallet() -> Keypair {
    let wallet_json_path = "/Users/l/Downloads/test_jz.json";
    let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
    wallet
}
