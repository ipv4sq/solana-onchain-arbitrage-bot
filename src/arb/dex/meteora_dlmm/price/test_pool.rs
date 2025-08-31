use crate::arb::dex::interface::PoolDataLoader;
use crate::arb::dex::meteora_dlmm::pool_data::MeteoraDlmmPoolData;
use crate::arb::global::state::rpc::rpc_client;
use crate::arb::util::traits::pubkey::ToPubkey;

#[tokio::test]
async fn inspect_trump_pool() {
    let pool_address = "9d9mb8kooFfaD3SctgZtkxQypkshx6ezhbKio89ixyy2".to_pubkey();
    let trump_mint = "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN".to_pubkey();
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();

    let account_data = rpc_client().get_account_data(&pool_address).await.unwrap();
    let pool_data = MeteoraDlmmPoolData::load_data(&account_data).unwrap();

    println!("\n=== TRUMP/USDC Pool Analysis ===");
    println!("Pool: {}", pool_address);
    println!("Token X: {}", pool_data.token_x_mint);
    println!("Token Y: {}", pool_data.token_y_mint);
    println!("TRUMP is: {}", if pool_data.token_x_mint == trump_mint { "Token X" } else { "Token Y" });
    println!("Active Bin ID: {}", pool_data.active_id);
    println!("Bin Step: {} bps", pool_data.bin_step);
    println!("Status: {}", pool_data.status);
    
    println!("\nBin Array Bitmap:");
    for (i, bitmap) in pool_data.bin_array_bitmap.iter().enumerate() {
        if *bitmap != 0 {
            println!("  Bitmap[{}]: 0x{:016x}", i, bitmap);
        }
    }
}