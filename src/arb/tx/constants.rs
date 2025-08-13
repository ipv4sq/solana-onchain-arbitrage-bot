use solana_program::pubkey::Pubkey;

// Known DEX program IDs
pub struct KnownPoolPrograms;

impl KnownPoolPrograms {
    pub const RAYDIUM_V4: &'static str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
    pub const RAYDIUM_CP: &'static str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
    pub const RAYDIUM_CLMM: &'static str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
    pub const PUMP: &'static str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
    pub const METEORA_DLMM: &'static str = "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo";
    pub const METEORA_DAMM: &'static str = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB";
    pub const METEORA_DAMM_V2: &'static str = "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG";
    pub const WHIRLPOOL: &'static str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
    pub const SOLFI: &'static str = "SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe";
    pub const VERTIGO: &'static str = "vrTGoBuy5rYSxAfV3jaRJWHH6nN9WK4NRExGxsk1bCJ";
}

// DEX types that can be identified in the transaction
#[derive(Debug, Clone, PartialEq)]
pub enum DexType {
    RaydiumV4,
    RaydiumCp,
    RaydiumClmm,
    Pump,
    MeteoraDlmm,
    MeteoraDamm,
    MeteoraDammV2,
    OrcaWhirlpool,
    Solfi,
    Vertigo,
    Unknown,
}

impl DexType {
    // Determine DEX type from a program ID
    pub fn determine_from(program_id: &Pubkey) -> Self {
        let program_str = program_id.to_string();
        
        match program_str.as_str() {
            KnownPoolPrograms::RAYDIUM_V4 => DexType::RaydiumV4,
            KnownPoolPrograms::RAYDIUM_CP => DexType::RaydiumCp,
            KnownPoolPrograms::RAYDIUM_CLMM => DexType::RaydiumClmm,
            KnownPoolPrograms::PUMP => DexType::Pump,
            KnownPoolPrograms::METEORA_DLMM => DexType::MeteoraDlmm,
            KnownPoolPrograms::METEORA_DAMM => DexType::MeteoraDamm,
            KnownPoolPrograms::METEORA_DAMM_V2 => DexType::MeteoraDammV2,
            KnownPoolPrograms::WHIRLPOOL => DexType::OrcaWhirlpool,
            KnownPoolPrograms::SOLFI => DexType::Solfi,
            KnownPoolPrograms::VERTIGO => DexType::Vertigo,
            _ => DexType::Unknown,
        }
    }
}
