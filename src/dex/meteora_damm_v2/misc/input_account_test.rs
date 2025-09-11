use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::meteora_damm_v2::misc::input_account::MeteoraDammV2InputAccount;
use crate::dex::meteora_damm_v2::misc::input_data::is_meteora_damm_v2_swap;
use crate::global::constant::pool_program::PoolProgram;
use crate::global::constant::token_program::TokenProgram;
use crate::program::mev_bot::ix::extract_mev_instruction;
use crate::util::traits::account_meta::ToAccountMeta;
use crate::util::traits::pubkey::ToPubkey;

fn expected_account() -> MeteoraDammV2InputAccount {
    MeteoraDammV2InputAccount {
        pool_authority: "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC".to_readonly(),
        pool: "6CXXieC355gteamwofSzJn8DiyrbKyYyXc3eBKmB81CF".to_writable(),
        input_token_account: "AaeZVRToQvmEBuU9EjypuYs3GyVZSZhKpCV2opPa4Biy".to_writable(),
        output_token_account: "Aiaz92F1keKEfJkfWjvRrp34D8Wh4dGRbrSDuHzV289s".to_writable(),
        token_a_vault: "9B3KPhHyDhUmNvjY2vk6JYs3vfUgPTzB9u1fWYsfK1s5".to_writable(),
        token_b_vault: "wAx8Her71ffN9hNyh5nj6WR7m56tAGrkajNiEdoGy4G".to_writable(),
        token_a_mint: "G1DXVVmqJs8Ei79QbK41dpgk2WtXSGqLtx9of7o8BAGS".to_readonly(),
        token_b_mint: "So11111111111111111111111111111111111111112".to_readonly(),
        payer: "4UX2dsCbqCm475cM2VvbEs6CmgoAhwP9CnwRT6WxmYA5".to_signer(),
        token_a_program: TokenProgram::SPL_TOKEN.to_program(),
        token_b_program: TokenProgram::SPL_TOKEN.to_program(),
        referral_token_program: PoolProgram::METEORA_DAMM_V2.to_program(),
        event_authority: "3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet".to_readonly(),
        meteora_program: PoolProgram::METEORA_DAMM_V2.to_program(),
    }
}

#[tokio::test]
async fn test_restore_from() {
    use crate::sdk::solana_rpc::client::_set_test_client;
    use crate::sdk::solana_rpc::methods::transaction::fetch_tx;
    
    // https://solscan.io/tx/57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP
    const TX: &str =
        "57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP";
    
    _set_test_client();
    let tx = fetch_tx(TX).await.unwrap();

    let (_, inner_ixs) = extract_mev_instruction(&tx).unwrap();

    // Find the actual swap instruction (the one with 14 accounts)
    let damm_v2_ix = inner_ixs
        .instructions
        .iter()
        .find(|ix| is_meteora_damm_v2_swap(&ix.data))
        .unwrap();

    let result = MeteoraDammV2InputAccount::restore_from(damm_v2_ix, &tx).unwrap();
    let expected = expected_account();
    assert_eq!(expected, result);
}

#[tokio::test]
async fn test_build_accounts() {
    use crate::dex::meteora_damm_v2::pool_data::test::load_pool_data;

    let pool = "6CXXieC355gteamwofSzJn8DiyrbKyYyXc3eBKmB81CF".to_pubkey();
    let pool_data = load_pool_data();
    let input_mint = "So11111111111111111111111111111111111111112".to_pubkey();
    let output_mint = "G1DXVVmqJs8Ei79QbK41dpgk2WtXSGqLtx9of7o8BAGS".to_pubkey();

    let result = MeteoraDammV2InputAccount::build_accounts_with_direction_and_size(
        &"4UX2dsCbqCm475cM2VvbEs6CmgoAhwP9CnwRT6WxmYA5".to_pubkey(),
        &pool,
        &pool_data,
        &input_mint,
        &output_mint,
        Some(3226352439),
        Some(0),
    )
    .await
    .unwrap();

    let expected = expected_account();
    assert_eq!(expected, result);
}
