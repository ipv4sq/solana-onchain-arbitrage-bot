use solana_program::instruction::AccountMeta;
use tracing::info;

pub fn log_account_metas(accounts: &[AccountMeta], context: &str) {
    info!("printing all the accounts for {}", context);
    accounts.iter().for_each(|account| {
        info!(
            "account: {}, signer: {}, writable: {}",
            account.pubkey, account.is_signer, account.is_writable
        )
    });
    info!("finished printing all the accounts for {}", context);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use solana_program::pubkey::Pubkey;

    fn new_unique_pubkey() -> Pubkey {
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        Pubkey::new_from_array(bytes)
    }

    #[test]
    fn test_log_account_metas() {
        let accounts = vec![
            AccountMeta::new(new_unique_pubkey(), true),
            AccountMeta::new_readonly(new_unique_pubkey(), false),
            AccountMeta::new(new_unique_pubkey(), false),
        ];

        log_account_metas(&accounts, "test context");

        assert_eq!(accounts[0].is_signer, true);
        assert_eq!(accounts[0].is_writable, true);
        assert_eq!(accounts[1].is_signer, false);
        assert_eq!(accounts[1].is_writable, false);
        assert_eq!(accounts[2].is_signer, false);
        assert_eq!(accounts[2].is_writable, true);
    }
}
