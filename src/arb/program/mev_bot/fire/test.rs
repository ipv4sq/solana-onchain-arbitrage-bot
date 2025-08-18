#[cfg(test)]
mod tests {
    use crate::arb::program::mev_bot::fire::construct::*;
    use crate::arb::chain::util::alt::fetch_address_lookup_tables;
    use crate::arb::constant::dex_type::DexType;
    use crate::constants::helpers::ToPubkey;
    use solana_sdk::hash::Hash;
    use solana_sdk::signature::read_keypair_file;
    use crate::arb::pool::register::AnyPoolConfig;
    use crate::arb::global::rpc::rpc_client;

    #[tokio::test]
    async fn test_build_tx() {
        let wallet_json_path = "/Users/l/Downloads/test_jz.json";
        let wallet = read_keypair_file(wallet_json_path).expect("Failed to read wallet keypair");
        let rpc_client = rpc_client();

        let gas_price = 1_000_000;
        let compute_unit_limit = 1_400_000;

        let meteora_dlmm_pool = "FoSDw2L5DmTuQTFe55gWPDXf88euaxAEKFre74CnvQbX".to_pubkey();
        let pool_config = AnyPoolConfig::from_address(&meteora_dlmm_pool, DexType::MeteoraDlmm)
            .await
            .expect("Failed to load pool config");

        let pools = vec![pool_config];

        let blockhash = Hash::default();

        let alt_keys = vec![
            "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC".to_pubkey(),
            "q52amtQzHcXs2PA3c4Xqv1LRRZCbFMzd4CGHu1tHdp1".to_pubkey(),
        ];

        let alts = fetch_address_lookup_tables(&rpc_client, &alt_keys)
            .await
            .expect("Failed to fetch ALTs");

        let tx = build_tx(
            &wallet,
            gas_price,
            compute_unit_limit,
            pools,
            blockhash,
            &alts,
            10_000,
        )
        .await;

        assert!(tx.is_ok(), "Failed to build transaction: {:?}", tx.err());

        let versioned_tx = tx.unwrap();
        assert_eq!(
            versioned_tx.signatures.len(),
            1,
            "Should have exactly one signature"
        );
        
        let tx_bytes = ::bincode::serialize(&versioned_tx).unwrap();
        let tx_size = tx_bytes.len();
        println!("Test: Transaction size = {} bytes", tx_size);
        assert!(tx_size > 0, "Transaction should have non-zero size");
        assert!(tx_size <= 1232, "Transaction should fit within Solana's 1232 byte limit");
    }
}