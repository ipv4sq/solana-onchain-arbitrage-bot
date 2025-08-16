// DEX types that can be identified in the transaction
#[derive(Debug, Clone, PartialEq, Copy)]
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

