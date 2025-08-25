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
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_log_account_metas() {
        let accounts = vec![
            AccountMeta::new(Pubkey::new_unique(), true),
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
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