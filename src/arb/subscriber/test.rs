mod test {
    use crate::arb::global::rpc::fetch_tx_sync;
    use crate::arb::program::solana_mev_bot::ix::extract_mev_instruction;
    use crate::arb::program::solana_mev_bot::subscriber::on_mev_bot_transaction;
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

        on_mev_bot_transaction(&tx, &ix, &inner).await.unwrap();
    }
}
