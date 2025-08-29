use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

pub struct MeteoraDAmmV2Info {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
}

impl MeteoraDAmmV2Info {
    pub fn load_checked(data: &[u8]) -> Result<Self> {
        let base_mint = Pubkey::new_from_array(data[168..200].try_into()?);
        let quote_mint = Pubkey::new_from_array(data[200..232].try_into()?);
        let base_vault = Pubkey::new_from_array(data[232..264].try_into()?);
        let quote_vault = Pubkey::new_from_array(data[264..296].try_into()?);
        Ok(Self {
            base_mint,
            quote_mint,
            base_vault,
            quote_vault,
        })
    }
}
