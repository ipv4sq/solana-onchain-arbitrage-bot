use crate::constants::helpers::ToPubkey;
use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;

pub struct KnownPoolPrograms;

impl KnownPoolPrograms {
    pub const RAYDIUM_V4: &'static str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
    pub const RAYDIUM_CPMM: &'static str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
    pub const RAYDIUM_CLMM: &'static str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
    pub const PUMP: &'static str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
    pub const METEORA_DLMM: &'static str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";
    pub const METEORA_DAMM: &'static str = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB";
    pub const METEORA_DAMM_V2: &'static str = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG";
    pub const WHIRLPOOL: &'static str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
    pub const SOLFI: &'static str = "SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe";
    pub const VERTIGO: &'static str = "vrTGoBuy5rYSxAfV3jaRJWHH6nN9WK4NRExGxsk1bCJ";
}

lazy_static! {
    pub static ref RAYDIUM_V4_PROGRAM: Pubkey = KnownPoolPrograms::RAYDIUM_V4.to_pubkey();
    pub static ref RAYDIUM_CPMM_PROGRAM: Pubkey = KnownPoolPrograms::RAYDIUM_CPMM.to_pubkey();
    pub static ref RAYDIUM_CLMM_PROGRAM: Pubkey = KnownPoolPrograms::RAYDIUM_CLMM.to_pubkey();
    pub static ref PUMP_PROGRAM: Pubkey = KnownPoolPrograms::PUMP.to_pubkey();
    pub static ref METEORA_DLMM_PROGRAM: Pubkey = KnownPoolPrograms::METEORA_DLMM.to_pubkey();
    pub static ref METEORA_DAMM_PROGRAM: Pubkey = KnownPoolPrograms::METEORA_DAMM.to_pubkey();
    pub static ref METEORA_DAMM_V2_PROGRAM: Pubkey = KnownPoolPrograms::METEORA_DAMM_V2.to_pubkey();
    pub static ref WHIRLPOOL_PROGRAM: Pubkey = KnownPoolPrograms::WHIRLPOOL.to_pubkey();
    pub static ref SOLFI_PROGRAM: Pubkey = KnownPoolPrograms::SOLFI.to_pubkey();
    pub static ref VERTIGO_PROGRAM: Pubkey = KnownPoolPrograms::VERTIGO.to_pubkey();
}
