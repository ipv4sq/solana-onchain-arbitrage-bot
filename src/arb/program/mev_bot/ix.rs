use crate::arb::convention::chain::instruction::{InnerInstructions, Instruction};
use crate::arb::convention::chain::Transaction;
use crate::arb::global::constant::mev_bot::MevBot;
use crate::arb::global::constant::token_program::TokenProgram;
use crate::arb::program::mev_bot::ix_input::{SolanaMevBotIxInput, SolanaMevBotIxInputData};
use anyhow::Result;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use std::collections::HashMap;
use tracing::error;

pub fn convert_to_smb_ix(ix: &Instruction) -> Result<SolanaMevBotIxInput> {
    let data = SolanaMevBotIxInputData::from_bytes(&ix.data)?;

    Ok(SolanaMevBotIxInput {
        program_id: ix.program_id,
        accounts: ix.accounts.clone(),
        data,
    })
}

pub fn extract_mev_instruction(tx: &Transaction) -> Option<(&Instruction, &InnerInstructions)> {
    tx.find_top_ix_interact_with(|program_id| *program_id == MevBot::EMV_BOT_PROGRAM)
}

#[derive(Debug, Clone)]
pub struct BalanceStatement {
    pub mint: Pubkey,
    pub amount: i64, // Using i64 to track net flow (positive = inflow, negative = outflow)
}

pub fn is_mev_box_ix_profitable(
    ix: &Instruction,
    inners: &InnerInstructions,
) -> Result<HashMap<Pubkey, Vec<BalanceStatement>>> {
    // Build ATA balances using functional chaining
    let ata_balances: HashMap<Pubkey, BalanceStatement> = inners
        .instructions
        .iter()
        .filter_map(|ix| ix.as_sol_token_transfer_checked())
        .fold(HashMap::new(), |mut acc, transfer| {
            // Update source account (outflow)
            acc.entry(transfer.source)
                .and_modify(|e| e.amount -= transfer.amount as i64)
                .or_insert(BalanceStatement {
                    mint: transfer.mint,
                    amount: -(transfer.amount as i64),
                });

            acc.entry(transfer.destination)
                .and_modify(|e| e.amount += transfer.amount as i64)
                .or_insert(BalanceStatement {
                    mint: transfer.mint,
                    amount: transfer.amount as i64,
                });

            acc
        });

    let potential_owners: Vec<Pubkey> = ix
        .accounts
        .iter()
        .map(|acc| (acc.pubkey, acc.is_signer))
        .fold(Vec::new(), |mut owners, (pubkey, is_signer)| {
            if is_signer && !owners.contains(&pubkey) {
                owners.insert(0, pubkey);
            } else if !owners.contains(&pubkey) {
                owners.push(pubkey);
            }
            owners
        });

    // Map ATAs to owner addresses using functional chaining
    let owner_balances: HashMap<Pubkey, HashMap<Pubkey, i64>> = ata_balances
        .iter()
        .filter_map(|(ata, balance)| {
            find_ata_owner(ata, &balance.mint, &potential_owners)
                .map(|owner| (owner, balance.mint, balance.amount))
                .or_else(|| {
                    error!(
                        "Failed to find owner for ATA: {} (mint: {}), skipping this balance",
                        ata, balance.mint
                    );
                    None
                })
        })
        .fold(HashMap::new(), |mut acc, (owner, mint, amount)| {
            acc.entry(owner)
                .or_insert_with(HashMap::new)
                .entry(mint)
                .and_modify(|amt| *amt += amount)
                .or_insert(amount);

            acc
        });

    // Convert to final format using functional chaining
    Ok(owner_balances
        .into_iter()
        .filter_map(|(owner, mints)| {
            let balances: Vec<BalanceStatement> = mints
                .into_iter()
                .filter(|(_, amount)| *amount != 0)
                .map(|(mint, amount)| BalanceStatement { mint, amount })
                .collect();

            (!balances.is_empty()).then_some((owner, balances))
        })
        .collect())
}

// Helper function to find the owner of an ATA by trying to derive it
fn find_ata_owner(ata: &Pubkey, mint: &Pubkey, potential_owners: &[Pubkey]) -> Option<Pubkey> {
    // Try both token programs
    let token_programs = [spl_token::ID, TokenProgram::TOKEN_2022];

    // Check each potential owner with each token program
    for owner in potential_owners {
        for token_program in &token_programs {
            let derived_ata =
                get_associated_token_address_with_program_id(owner, mint, token_program);

            if &derived_ata == ata {
                return Some(*owner);
            }
        }
    }

    // If no match found in potential owners, we could try some well-known addresses
    // or return None to indicate we couldn't determine the owner
    None
}

#[cfg(test)]
mod tests {
    use crate::arb::global::client::rpc::fetch_tx;
    use crate::arb::global::constant::mint::Mints;
    use crate::arb::program::mev_bot::ix::{extract_mev_instruction, is_mev_box_ix_profitable};
    use crate::arb::util::traits::pubkey::ToPubkey;

    #[tokio::test]
    async fn test_account_metadata_mapping() {
        let tx_hash = "3mDkuLRaZRuGDcHon9JFGikkb7YQnc8Ph4NBjUG1vrbWLpCDvgMbHMDFycvtvwQv6BU2aF6wQbmQjdVNzHRGTQKs";
        let tx = fetch_tx(tx_hash).await.unwrap();
        let (ix, inner) = extract_mev_instruction(&tx).unwrap();

        // Find a transfer_checked instruction in the inner instructions
        let transfer_checked_ix = inner
            .instructions
            .iter()
            .find(|ix| {
                ix.program_id == spl_token::ID
                    && ix.accounts.len() == 4
                    && !ix.data.is_empty()
                    && ix.data[0] == 12 // transfer_checked discriminator
            })
            .expect("Should find at least one transfer_checked instruction");

        println!("Transfer checked instruction account metadata:");
        for (i, acc) in transfer_checked_ix.accounts.iter().enumerate() {
            println!(
                "  Account {}: pubkey={}, is_signer={}, is_writable={}",
                i, acc.pubkey, acc.is_signer, acc.is_writable
            );
        }

        // The expected metadata for transfer_checked should be:
        // 0: source (writable, not signer)
        // 1: mint (not writable, not signer)
        // 2: destination (writable, not signer)
        // 3: authority (not writable, signer OR not signer for PDA)

        // Let's check if the metadata makes sense
        assert_eq!(
            transfer_checked_ix.accounts.len(),
            4,
            "transfer_checked should have 4 accounts"
        );

        // For debugging - let's not assert on writable/signer yet, just observe
        println!("\nExpected vs Actual:");
        println!(
            "Account 0 (source): should be writable=true, is_writable={}",
            transfer_checked_ix.accounts[0].is_writable
        );
        println!(
            "Account 1 (mint): should be writable=false, is_writable={}",
            transfer_checked_ix.accounts[1].is_writable
        );
        println!(
            "Account 2 (dest): should be writable=true, is_writable={}",
            transfer_checked_ix.accounts[2].is_writable
        );
    }

    #[tokio::test]
    /*
    Copied from solscan, for claude code to create test.
    1. swap 7.107544925 wsol -> 1,684,417.981584314 meme coin
    2. swap 1,684,417.981584314 wsol -> 7.343898162 wsol
    result  +0.236353237 sol after this arbitrage
    beneficial owner is 9FEjMA5uSKMWkLpaXJQY7V4nLm2xvvMxkeyeGEi7SLEg
     */
    async fn test_is_mev_box_ix_profitable() {
        let tx_hash = "3mDkuLRaZRuGDcHon9JFGikkb7YQnc8Ph4NBjUG1vrbWLpCDvgMbHMDFycvtvwQv6BU2aF6wQbmQjdVNzHRGTQKs";
        let tx = fetch_tx(tx_hash).await.unwrap();
        let (ix, inner) = extract_mev_instruction(&tx).unwrap();
        let result = is_mev_box_ix_profitable(&ix, &inner).unwrap();

        // Expected beneficial owner
        let expected_owner = "9FEjMA5uSKMWkLpaXJQY7V4nLm2xvvMxkeyeGEi7SLEg".to_pubkey();
        let wsol_mint = Mints::WSOL;

        // Expected profit in lamports (0.236353237 SOL = 236353237 lamports)
        let expected_profit_lamports = 236353237i64;

        // Check that the expected owner exists in results
        assert!(
            result.contains_key(&expected_owner),
            "Expected owner {} not found in results. Found owners: {:?}",
            expected_owner,
            result.keys().collect::<Vec<_>>()
        );

        // Get the balance statements for the owner
        let owner_balances = result.get(&expected_owner).unwrap();

        // Find WSOL balance
        let wsol_balance = owner_balances
            .iter()
            .find(|b| b.mint == wsol_mint)
            .expect("WSOL balance not found for owner");

        // Verify the profit amount (allowing small rounding differences)
        let profit_difference = (wsol_balance.amount - expected_profit_lamports).abs();
        assert!(
            profit_difference < 1000, // Allow up to 1000 lamports difference for rounding
            "Unexpected WSOL profit. Expected: {} lamports, Got: {} lamports, Difference: {} lamports",
            expected_profit_lamports,
            wsol_balance.amount,
            profit_difference
        );

        // Verify profit is positive
        assert!(
            wsol_balance.amount > 0,
            "WSOL balance should be positive (profit), got: {}",
            wsol_balance.amount
        );
    }
}
