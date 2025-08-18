#[cfg(test)]
mod tests {
    use crate::arb::constant::dex_type::DexType;
    use crate::arb::constant::mint::Mints;
    use crate::arb::global::rpc::ensure_mint_account_exists;
    use crate::arb::pool::register::AnyPoolConfig;
    use crate::arb::program::mev_bot::fire::construct::*;
    use crate::constants::helpers::ToPubkey;
    use solana_sdk::signature::{read_keypair_file, Keypair};

    fn get_wallet() -> Keypair {
        let wallet_json_path = "/Users/l/Downloads/test_jz.json";
        let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
        return wallet;
    }

    #[tokio::test]
    async fn create_wsol_ata() {
        // let result = ensure_mint_account_exists(&Mints::WSOL.to_pubkey(), &get_wallet()).await;
        let _ = ensure_mint_account_exists(
            &"9yBQVHj2FJnf7XfQWUPQoj3iyMwAXQMxBWD37cwFBAGS".to_pubkey(),
            &get_wallet(),
        ).await;
    }

    #[tokio::test]
    async fn test_build_tx() {
        let wallet_json_path = "/Users/l/Downloads/test_jz.json";
        let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
        let unit_price = 10_000;
        let compute_unit_limit = 400_000;
        let meteora_dlmm_pool = "3odMjqSfsfj9uGHg7Ax4UWmiayCzQXZn6gNpmuxpttSk".to_pubkey();
        let meteora_damm_v2_pool = "G2TGspLi4G1LfH8ExkiMNS5mCZsgKvKtSBP6rNwMavd9".to_pubkey();
        let result = build_and_send(
            &wallet,
            compute_unit_limit,
            unit_price,
            vec![
                AnyPoolConfig::from_address(&meteora_dlmm_pool, DexType::MeteoraDlmm)
                    .await
                    .expect("Failed to load pool config"),
                AnyPoolConfig::from_address(&meteora_damm_v2_pool, DexType::MeteoraDammV2)
                    .await
                    .expect("Failed to load pool config"),
            ],
            1000,
        )
        .await;
        println!("{:?}", result);
    }
}
