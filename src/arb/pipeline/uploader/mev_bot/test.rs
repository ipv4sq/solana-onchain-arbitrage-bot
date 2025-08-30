#[cfg(test)]
mod tests {
    use crate::arb::dex::any_pool_config::AnyPoolConfig;

    use crate::arb::global::enums::dex_type::DexType;
    use crate::arb::global::state::rpc::{ensure_mint_account_exists, fetch_tx};
    use crate::arb::pipeline::uploader::mev_bot::construct::*;
    use crate::arb::program::mev_bot::ix::extract_mev_instruction;
    use crate::arb::util::traits::pubkey::ToPubkey;

    use solana_program::pubkey::Pubkey;
    use solana_sdk::signature::{read_keypair_file, Keypair};

    use crate::arb::global::constant::token_program::TokenProgram;
    use crate::arb::global::trace::types::Trace;
    use crate::arb::pipeline::uploader::entry::build_and_send;
    use crate::arb::util::debug::log_account_metas;

    fn get_wallet() -> Keypair {
        let wallet_json_path = "/Users/l/Downloads/test_jz.json";
        let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
        return wallet;
    }

    fn minor_mint() -> Pubkey {
        "9yBQVHj2FJnf7XfQWUPQoj3iyMwAXQMxBWD37cwFBAGS".to_pubkey()
    }

    #[tokio::test]
    async fn create_wsol_ata() {
        // let result = ensure_mint_account_exists(&Mints::WSOL.to_pubkey(), &get_wallet()).await;
        let _ = ensure_mint_account_exists(&minor_mint(), &get_wallet()).await;
    }

    #[tokio::test]
    async fn example_tx() {
        let tx = "5xt23Gje9fJJH3EA2etsgfGkkdAKHMaWexeGo6BrzbXRZWVvrxr6erazi4wPSYFq7eyZSvE7kJXF86rTYDDJV4F3";
        let transaction = fetch_tx(&tx).await.unwrap();
        let (ix, _) =
            extract_mev_instruction(&transaction).expect("Failed to extract MEV instruction");
        log_account_metas(&ix.accounts, "in test");
    }
}
