use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

pub mod config;
pub mod misc;
pub mod pool_data;
mod price_calculator;

pub static PUMP_GLOBAL_CONFIG: Pubkey = pubkey!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");
