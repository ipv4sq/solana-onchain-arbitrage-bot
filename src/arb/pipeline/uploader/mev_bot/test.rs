#[cfg(test)]
mod tests {
    use crate::arb::convention::pool::register::AnyPoolConfig;

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
    async fn test_send_tx() {
        let wallet_json_path = "/Users/l/Downloads/test_jz.json";
        let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
        let unit_price = 10_000;
        let compute_unit_limit = 400_000;
        let meteora_dlmm_pool = "3odMjqSfsfj9uGHg7Ax4UWmiayCzQXZn6gNpmuxpttSk".to_pubkey();
        let meteora_damm_v2_pool = "G2TGspLi4G1LfH8ExkiMNS5mCZsgKvKtSBP6rNwMavd9".to_pubkey();
        let pools = vec![
            AnyPoolConfig::from_address(&meteora_dlmm_pool, DexType::MeteoraDlmm)
                .await
                .expect("Failed to load pool config"),
            AnyPoolConfig::from_address(&meteora_damm_v2_pool, DexType::MeteoraDammV2)
                .await
                .expect("Failed to load pool config"),
        ];
        let result = build_and_send(
            &wallet,
            &minor_mint(),
            compute_unit_limit,
            unit_price,
            &pools,
            1000,
            false,
            Trace::new(),
        )
        .await;
        println!("{:?}", result.unwrap().0);
    }

    #[tokio::test]
    async fn example_tx() {
        let tx = "5xt23Gje9fJJH3EA2etsgfGkkdAKHMaWexeGo6BrzbXRZWVvrxr6erazi4wPSYFq7eyZSvE7kJXF86rTYDDJV4F3";
        let transaction = fetch_tx(&tx).await.unwrap();
        let (ix, _) =
            extract_mev_instruction(&transaction).expect("Failed to extract MEV instruction");
        log_account_metas(&ix.accounts, "in test");
    }

    #[tokio::test]
    async fn reproduce() {
        let meteora_dlmm_pool = "3odMjqSfsfj9uGHg7Ax4UWmiayCzQXZn6gNpmuxpttSk".to_pubkey();
        let meteora_damm_v2_pool = "G2TGspLi4G1LfH8ExkiMNS5mCZsgKvKtSBP6rNwMavd9".to_pubkey();
        let pools = vec![
            AnyPoolConfig::from_address(&meteora_dlmm_pool, DexType::MeteoraDlmm)
                .await
                .expect("Failed to load pool config"),
            AnyPoolConfig::from_address(&meteora_damm_v2_pool, DexType::MeteoraDammV2)
                .await
                .expect("Failed to load pool config"),
        ];
        let ix = create_invoke_mev_instruction(
            &"DvLTm5iR43m7u2Rh5rwNmwrKDtD9X8iHpaoLhaUnEKEq".to_pubkey(),
            &minor_mint(),
            &TokenProgram::SPL_TOKEN,
            1,
            &pools,
            1000,
        )
        .unwrap();
    }
}
