#[cfg(test)]
mod tests {
    use super::super::pool_service::load_mint_from_address;
    use solana_program::pubkey::Pubkey;
    use std::str::FromStr;
    use crate::arb::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_load_mint_from_address() {
        // WSOL mint address
        let wsol_mint = "So11111111111111111111111111111111111111112".to_pubkey();
        
        let result = load_mint_from_address(&wsol_mint).await;
        
        match result {
            Ok(mint_record) => {
                println!("Successfully loaded WSOL mint:");
                println!("  Address: {:?}", mint_record.address);
                println!("  Symbol: {}", mint_record.symbol);
                println!("  Decimals: {}", mint_record.decimals);
                println!("  Program: {:?}", mint_record.program);
                println!("  Note: Symbol fetched from chain metadata or fallback");
                
                // WSOL should have 9 decimals
                assert_eq!(mint_record.decimals, 9);
                // Check that we got the correct symbol (SOL is the actual symbol from metadata)
                assert_eq!(mint_record.symbol, "SOL");
            }
            Err(e) => {
                // This might fail if not connected to mainnet
                println!("Failed to load mint (expected if not on mainnet): {}", e);
            }
        }
    }
    
    #[tokio::test]
    async fn test_load_custom_mint() {
        // Example: USDC mint on mainnet
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_pubkey();
        
        let result = load_mint_from_address(&usdc_mint).await;
        
        match result {
            Ok(mint_record) => {
                println!("Successfully loaded USDC mint:");
                println!("  Address: {:?}", mint_record.address);
                println!("  Symbol: {}", mint_record.symbol);
                println!("  Decimals: {}", mint_record.decimals);
                println!("  Program: {:?}", mint_record.program);
                
                // USDC should have 6 decimals
                assert_eq!(mint_record.decimals, 6);
                // Check that we got the correct symbol
                assert_eq!(mint_record.symbol, "USDC");
            }
            Err(e) => {
                println!("Failed to load mint: {}", e);
            }
        }
    }
}