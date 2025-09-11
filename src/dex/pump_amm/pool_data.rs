use crate::dex::interface::PoolDataLoader;
use crate::global::constant::pool_program::PoolProgram;
use crate::util::serde_helpers;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[repr(C)]
pub struct PumpAmmPoolData {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    #[serde(with = "serde_helpers::byte_array_57")]
    pub _padding: [u8; 57],
}

impl PoolDataLoader for PumpAmmPoolData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        PumpAmmPoolData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn base_mint(&self) -> Pubkey {
        self.base_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.quote_mint
    }

    fn base_vault(&self) -> Pubkey {
        self.pool_base_token_account
    }

    fn quote_vault(&self) -> Pubkey {
        self.pool_quote_token_account
    }
}

impl PumpAmmPoolData {
    pub(crate) fn get_creator_vault_authority(coin_creator: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[b"creator_vault", coin_creator.as_ref()],
            &PoolProgram::PUMP,
        )
        .0
    }

    fn get_creator_vault_ata(vault_authority: &Pubkey, token_mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(vault_authority, token_mint)
    }
}

#[allow(non_upper_case_globals)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::solana_rpc::proxy;
    use crate::util::traits::pubkey::ToPubkey;
    use serde_json::Value;

    static PoolAddress: &str = "8uAAT95mo699fJ6CMpRw28DKfeVudGkonhEgmNPAEmCE";
    static PoolDataJson: &str = r#"
{"pool_bump":{"type":"u8","data":254},"index":{"type":"u16","data":"0"},"creator":{"type":"pubkey","data":"F2JaD6abvdVmbbUFcHQEiAXSYWEc18nDFcsctiLqv8xR"},"base_mint":{"type":"pubkey","data":"GNHW5JetZmW85vAU35KyoDcYoSd3sNWtx5RPMTDJpump"},"quote_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"lp_mint":{"type":"pubkey","data":"6QerJjW7mQjU2dXttwzYgZXh1EUo3SZFZr8jM1HSPF93"},"pool_base_token_account":{"type":"pubkey","data":"BwUJWqSSQEyTxgYMzrvfRjgnjvLghrCDZbSAp483bX6D"},"pool_quote_token_account":{"type":"pubkey","data":"G85kUJohot7w9RB3LyVRP4tS2kVRqm6PnU4XK83GxfN3"},"lp_supply":{"type":"u64","data":"4193687021413"},"coin_creator":{"type":"pubkey","data":"GJ7mfMrEeYs3rjyN7Ed2UyJ6FApvQhuuEX5yZCWkZU8V"}}
    "#;

    #[tokio::test]
    async fn test_pool_data_matches_solscan() {
        let pool_address = PoolAddress.to_pubkey();
        let account = proxy::get_account(&pool_address)
            .await
            .expect("Failed to fetch pool account from RPC");

        let pool_data_from_rpc =
            PumpAmmPoolData::load_data(&account.data).expect("Failed to load pool data from RPC");

        let json_data: Value =
            serde_json::from_str(PoolDataJson).expect("Failed to parse JSON data");

        assert_eq!(
            pool_data_from_rpc.pool_bump,
            json_data["pool_bump"]["data"].as_u64().unwrap() as u8
        );

        assert_eq!(
            pool_data_from_rpc.index,
            json_data["index"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );

        assert_eq!(
            pool_data_from_rpc.creator,
            json_data["creator"]["data"].as_str().unwrap().to_pubkey()
        );

        assert_eq!(
            pool_data_from_rpc.base_mint,
            json_data["base_mint"]["data"].as_str().unwrap().to_pubkey()
        );

        assert_eq!(
            pool_data_from_rpc.quote_mint,
            json_data["quote_mint"]["data"]
                .as_str()
                .unwrap()
                .to_pubkey()
        );

        assert_eq!(
            pool_data_from_rpc.lp_mint,
            json_data["lp_mint"]["data"].as_str().unwrap().to_pubkey()
        );

        assert_eq!(
            pool_data_from_rpc.pool_base_token_account,
            json_data["pool_base_token_account"]["data"]
                .as_str()
                .unwrap()
                .to_pubkey()
        );

        assert_eq!(
            pool_data_from_rpc.pool_quote_token_account,
            json_data["pool_quote_token_account"]["data"]
                .as_str()
                .unwrap()
                .to_pubkey()
        );

        assert_eq!(
            pool_data_from_rpc.lp_supply,
            json_data["lp_supply"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );

        assert_eq!(
            pool_data_from_rpc.coin_creator,
            json_data["coin_creator"]["data"]
                .as_str()
                .unwrap()
                .to_pubkey()
        );
    }
}
