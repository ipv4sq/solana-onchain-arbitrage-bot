use crate::global::constant::token_program::TokenProgram;
use crate::sdk::solana_rpc::rpc;
use crate::util::solana::pda::ata;
use solana_program::pubkey::Pubkey;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{Keypair, Signer};
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use tracing::info;

pub async fn ensure_mint_account_exists(mint: &Pubkey, wallet: &Keypair) -> anyhow::Result<bool> {
    let owner = &wallet.pubkey();
    let mint_owner = rpc::rpc_client().get_account(mint).await?.owner;
    if mint_owner != TokenProgram::TOKEN_2022 && mint_owner != TokenProgram::SPL_TOKEN {
        return Err(anyhow::anyhow!(
            "mint owner should be Token2022 or SPL Token program but instead it's: {}",
            mint_owner
        ));
    }

    let mint_account = ata(owner, mint, &mint_owner);
    let mint_account_exists = rpc::rpc_client().get_account(&mint_account).await.is_ok();
    if !mint_account_exists {
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[
                ComputeBudgetInstruction::set_compute_unit_price(1_000),
                ComputeBudgetInstruction::set_compute_unit_limit(30_000),
                create_associated_token_account_idempotent(owner, owner, mint, &mint_owner),
            ],
            Some(owner),
            &[wallet],
            rpc::rpc_client().get_latest_blockhash().await?,
        );
        let signature = rpc::rpc_client()
            .send_and_confirm_transaction_with_spinner(&tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;
        info!(
            "Created token account for mint: {} owner: {} tx: {}",
            mint, owner, signature
        );
    } else {
        info!("Mint account exists mint: {} owner: {}", mint, owner);
    }
    Ok(true)
}
