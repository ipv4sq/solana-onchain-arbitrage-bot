use rust_decimal::Decimal;

pub mod any_pool_config;
pub mod interface;
pub mod legacy_interface;
pub mod meteora_damm;
pub mod meteora_damm_v2;
pub mod meteora_dlmm;
pub mod pump_amm;
pub mod raydium_cl_amm;
pub mod raydium_cpmm;
mod verification;
pub mod whirlpool;

#[derive(Debug, Clone, Copy)]
pub struct EstimatedQuote {
    pub mid_price: Decimal,
}
