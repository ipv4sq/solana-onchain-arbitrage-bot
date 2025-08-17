use crate::arb::chain::util::transaction::inner_to_filtered_map;
use crate::arb::constant::dex_type::DexType;
use crate::arb::constant::pool_owner::PoolOwnerPrograms;
use crate::arb::global::rpc::fetch_tx_sync;
use crate::arb::pool::register::AnyPoolConfig;
use crate::arb::program::mev_bot::ix::{convert_to_smb_ix, extract_mev_instruction};
use crate::test::test_utils::get_test_rpc_client;

mod test {
    use crate::arb::global::rpc::fetch_tx_sync;
    use crate::arb::program::mev_bot::ix::extract_mev_instruction;
    use crate::arb::program::mev_bot::onchain_monitor::entry::entry;
    use crate::test::test_utils::get_test_rpc_client;

    #[tokio::test]
    async fn test_on_mev_bot_transaction() {
        let sig = "57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP";

        let tx = tokio::task::spawn_blocking(move || {
            let client = get_test_rpc_client();
            fetch_tx_sync(&client, sig).unwrap()
        })
        .await
        .unwrap();

        let (ix, inner) = extract_mev_instruction(&tx).unwrap();

        entry(&tx).await.unwrap();
    }
}

#[test]
fn test_modular_functions() {
    let client = get_test_rpc_client();
    let sig =
        "2GNmMyHst1qd9B6FLAwBqrD6VdpxzLVxTZBuNSGYHt3Y5KtX93W6WWZGbsTfKKkbZcGi1M4KZRPQcev2VNpxLyck";
    let tx = fetch_tx_sync(&client, sig).expect("Failed to fetch transaction");
    let (raw_instruction, inner_ixs) =
        extract_mev_instruction(&tx).expect("Failed to extract MEV instruction");
    let parsed = convert_to_smb_ix(raw_instruction).expect("Failed to parse raw instruction");

    assert_eq!(parsed.data.instruction_discriminator, 28);
    assert_eq!(parsed.data.minimum_profit, 253345);
    assert_eq!(parsed.data.compute_unit_limit, 580000);
    assert_eq!(parsed.data.no_failure_mode, false);
    assert_eq!(parsed.data.use_flashloan, true);
    assert_eq!(parsed.accounts.len(), 59);
    assert!(inner_ixs.instructions.len() > 0);

    let swap_ixs = inner_to_filtered_map(inner_ixs);
    assert!(!swap_ixs.is_empty());

    for (program_id, ix) in swap_ixs.iter() {
        if program_id == PoolOwnerPrograms::METEORA_DLMM && ix.accounts.len() >= 15 {
            let swap_ix =
                AnyPoolConfig::from_ix(ix, &tx).expect("Failed to parse swap instruction");
            assert_eq!(swap_ix.dex_type, DexType::MeteoraDlmm);
            assert!(swap_ix.accounts.len() >= 15);
        }
    }
}
