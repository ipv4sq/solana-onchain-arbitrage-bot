use solana_client::rpc_client::RpcClient;

pub fn get_test_rpc_client() -> RpcClient {
    let free_alchemy_url =
        "https://solana-mainnet.g.alchemy.com/v2/FF6pAeQNXoud_0tmQI-auOtEG_ogMyRs";
    RpcClient::new(free_alchemy_url.to_string())
}

pub mod pool_addresses {
    pub const PUMP_TEST_POOL: &str = "7USDHmdsFsJGsrvuYWvYHKejJBneCLVk8hdMWVvb7VqA";
    pub const PUMP_TEST_TOKEN_MINT: &str = "34HDZNbUkTyTrgYKy2ox43yp2f8PJ5hoM7xsrfNApump";
}
