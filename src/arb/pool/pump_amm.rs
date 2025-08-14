use crate::arb::pool::interface::{PoolAccountDataLoader, PoolConfig, PoolConfigInit};
use crate::arb::constant::known_pool_program::PUMP_PROGRAM;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct PumpAmmAccountData {
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
    pub _padding: [u8; 57],
}

impl PoolAccountDataLoader for PumpAmmAccountData {
    fn load_data(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!(
                "Account data too short, expected at least 8 bytes"
            ));
        }

        PumpAmmAccountData::try_from_slice(&data[8..])
            .map_err(|e| anyhow::anyhow!("Failed to parse account data: {}", e))
    }

    fn get_base_mint(&self) -> Pubkey {
        self.base_mint
    }

    fn get_quote_mint(&self) -> Pubkey {
        self.quote_mint
    }

    fn get_base_vault(&self) -> Pubkey {
        self.pool_base_token_account
    }

    fn get_quote_vault(&self) -> Pubkey {
        self.pool_quote_token_account
    }
}

type PumpAmmPoolConfig = PoolConfig<PumpAmmAccountData>;

pub struct PumpAmmAccountSwapAccounts {}

impl PoolConfigInit<PumpAmmAccountData, PumpAmmAccountSwapAccounts> for PumpAmmPoolConfig {
    fn init(
        pool: &Pubkey,
        account_data: PumpAmmAccountData,
        desired_mint: Pubkey,
    ) -> Result<Self> {
        account_data.shall_contain(&desired_mint)?;

        Ok(PumpAmmPoolConfig {
            pool: *pool,
            data: account_data,
            desired_mint,
            minor_mint: account_data.the_other_mint(&desired_mint)?,
            // readonly_accounts: vec![
            //     desired_mint,
            //     *PUMP_PROGRAM,
            // ],
            // partial_writeable_accounts: vec![
            //     *pool,
            //     account_data.pool_base_token_account,
            //     account_data.pool_quote_token_account,
            //     PumpAmmAccountData::get_creator_vault_ata(
            //         &PumpAmmAccountData::get_creator_vault_authority(&account_data.coin_creator),
            //         &account_data.base_mint,
            //     ),
            // ],
        })
    }

    fn build_accounts(&self, payer: &Pubkey, input_mint: &Pubkey, output_mint: &Pubkey) -> Result<PumpAmmAccountSwapAccounts> {
        todo!()
    }
}

impl PumpAmmAccountData {
    fn get_creator_vault_authority(coin_creator: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[b"creator_vault", coin_creator.as_ref()],
            &*PUMP_PROGRAM,
        )
        .0
    }

    fn get_creator_vault_ata(vault_authority: &Pubkey, token_mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(vault_authority, token_mint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::helpers::ToPubkey;

    #[test]
    fn test_get_creator_vault_authority() {
        let expected = "wYGQPcWwnpSV1eqLQwatefR9ihwUtWoybNZcg8uvQWU";
        let coin_creator = "GeUnv1jmtviRbR7Gu1JnXSGkUMUgFVBHuEVQVpTaUX1W".to_pubkey();
        assert_eq!(
            PumpAmmAccountData::get_creator_vault_authority(&coin_creator),
            expected.to_pubkey()
        )
    }

    #[test]
    fn test_parse_pump_amm_account_data() {
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::Value;

        let data = general_purpose::STANDARD
            .decode(ACCOUNT_DATA_BASE64)
            .unwrap();
        let account = PumpAmmAccountData::load_data(&data).expect("Failed to parse account data");

        let json: Value = serde_json::from_str(ACCOUNT_DATA_JSON).expect("Failed to parse JSON");

        assert_eq!(
            account.pool_bump,
            json["pool_bump"]["data"]
                .as_u64()
                .unwrap() as u8
        );
        assert_eq!(
            account.index,
            json["index"]["data"]
                .as_str()
                .unwrap()
                .parse::<u16>()
                .unwrap()
        );
        assert_eq!(
            account.lp_supply,
            json["lp_supply"]["data"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );

        assert_eq!(
            account.creator.to_string(),
            json["creator"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.base_mint.to_string(),
            json["base_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.quote_mint.to_string(),
            json["quote_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.lp_mint.to_string(),
            json["lp_mint"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.pool_base_token_account.to_string(),
            json["pool_base_token_account"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.pool_quote_token_account.to_string(),
            json["pool_quote_token_account"]["data"].as_str().unwrap()
        );
        assert_eq!(
            account.coin_creator.to_string(),
            json["coin_creator"]["data"].as_str().unwrap()
        );
    }

    static POOL_ADDRESS: &str = "GUXAutvXh2Cvv2avGkbY8CfcsN9v2Uiwr8VCCqpn9HiU";
    static ACCOUNT_DATA_BASE64: &str = "8ZptBBGxbbz/AAAs98JusIgYjN/c+OImcGGKLbklh9KFdMPdkckEOMtRRiRJ+U7BLbz0edwLteAk1ZOEusHMAqkS9JpDqKYXM3n5BpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEIy+Mec8oHNW4ats6+SDWH6uwUdvnAkxqo7K1PvyTB1HQFfSZ1kp+pDQsMXajk+o6uiopINyd6LPEjQWvMErx5qIq9uP3CEJTTRxA+SM03rI36/NlFGPup8uXrVy0f5uRvp2tZ0AMAAOh5N8loi43Vzzi7J8beMK5OsQsmLpxQq4JX1IbnINydAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    static ACCOUNT_DATA_JSON: &str = r#"
{"pool_bump":{"type":"u8","data":255},"index":{"type":"u16","data":"0"},"creator":{"type":"pubkey","data":"42Y4PfN8eHhZJbp5vfqc59sMhw85g1QSdQ4WNYQgkgcM"},"base_mint":{"type":"pubkey","data":"3Sf6oKCeEqCuco4aYtKHHDTBYLAWHiL47QjvkW1UYDEG"},"quote_mint":{"type":"pubkey","data":"So11111111111111111111111111111111111111112"},"lp_mint":{"type":"pubkey","data":"bLafbwgNhmVvqLuxQWVrFY7QiwTU5FWNtd21WzAwxAo"},"pool_base_token_account":{"type":"pubkey","data":"8ou9YbVwLfMRXFq1ejqJcdXX9GM1bSAZUKrYzox3bSvc"},"pool_quote_token_account":{"type":"pubkey","data":"CLvCQE8qTSPmHJrxDqyrwP5hCvvuoha8tu7jY7bhxHFZ"},"lp_supply":{"type":"u64","data":"4193388308335"},"coin_creator":{"type":"pubkey","data":"GeUnv1jmtviRbR7Gu1JnXSGkUMUgFVBHuEVQVpTaUX1W"}}
"#;
}
