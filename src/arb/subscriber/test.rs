mod test {
    use crate::arb::subscriber::entry::on_mev_bot_transaction;
    use crate::arb::tx::tx_parser::{extract_mev_instruction, get_tx_by_sig};
    use crate::test::test_utils::get_test_rpc_client;

    #[tokio::test]
    async fn test_on_mev_bot_transaction() {
        let sig = "57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP";
        let client = get_test_rpc_client();
        let tx = get_tx_by_sig(&client, sig).unwrap();
        let (ix, inner) = extract_mev_instruction(&tx).unwrap();

        on_mev_bot_transaction(&tx, ix, inner).await.unwrap();
    }
}
