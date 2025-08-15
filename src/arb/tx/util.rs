use crate::constants::helpers::ToAccountMeta;
use anyhow::Result;
use solana_program::instruction::AccountMeta;
use solana_transaction_status::parse_accounts::ParsedAccount;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiMessage,
    UiPartiallyDecodedInstruction,
};

pub fn get_parsed_accounts(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<&Vec<ParsedAccount>> {
    match &tx.transaction.transaction {
        EncodedTransaction::Json(t) => match &t.message {
            UiMessage::Parsed(msg) => Ok(&msg.account_keys),
            _ => Err(anyhow::anyhow!("Transaction message is not parsed format")),
        },
        _ => Err(anyhow::anyhow!("Transaction is not in JSON format")),
    }
}

pub fn create_account_meta(
    parsed_accounts: &[ParsedAccount],
    ix: &UiPartiallyDecodedInstruction,
    index: usize,
) -> Result<AccountMeta> {
    let account_key = ix
        .accounts
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("Missing account at index {}", index))?;

    let parsed_acc = parsed_accounts
        .iter()
        .find(|acc| &acc.pubkey == account_key)
        .ok_or_else(|| anyhow::anyhow!("Account {} not found in parsed accounts", account_key))?;

    Ok(if parsed_acc.signer {
        account_key.to_signer()
    } else if parsed_acc.writable {
        account_key.to_writable()
    } else {
        account_key.to_readonly()
    })
}
