use anyhow::anyhow;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;

pub fn expect_owner(
    account_address: &Pubkey,
    account: &Account,
    expected: &Pubkey,
) -> anyhow::Result<()> {
    if account.owner != *expected {
        return Err(anyhow!(
            "Owner mismatch for account {account_address}: expected {expected}, got {}",
            account.owner
        ));
    }
    Ok(())
}
