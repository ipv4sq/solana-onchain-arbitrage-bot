use std::collections::HashMap;
use crate::arb::constant::pool_owner::{PoolOwnerPrograms, RECOGNIZED_POOL_OWNER_PROGRAMS};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction, UiParsedInstruction, UiPartiallyDecodedInstruction};
use crate::arb::chain::types::SwapInstruction;

pub fn is_meteora_damm_v2_swap(ix: &UiInstruction) -> Option<&UiPartiallyDecodedInstruction> {
    // METEORA_DAMM_V2 swap instructions have exactly 14 accounts
    is_program_ix(ix, PoolOwnerPrograms::METEORA_DAMM_V2, Some(14))
        .filter(|decoded| decoded.accounts.len() == 14)
}
pub fn is_meteora_dlmm_swap(ix: &UiInstruction) -> Option<&UiPartiallyDecodedInstruction> {
    is_program_ix(ix, PoolOwnerPrograms::METEORA_DLMM, Some(14))
        .filter(|decoded| decoded.accounts.len() > 14)
}

pub fn is_program_ix<'a>(
    ix: &'a UiInstruction,
    program_id: &str,
    min_accounts: Option<usize>,
) -> Option<&'a UiPartiallyDecodedInstruction> {
    if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(decoded)) = ix {
        if decoded.program_id == program_id {
            if let Some(min) = min_accounts {
                if decoded.accounts.len() >= min {
                    return Some(decoded);
                }
            } else {
                return Some(decoded);
            }
        }
    }
    None
}

pub fn extract_swap_inner_ix(
    inner_instructions: &UiInnerInstructions,
) -> HashMap<String, &UiPartiallyDecodedInstruction> {
    inner_instructions
        .instructions
        .iter()
        .filter_map(|x| match x {
            UiInstruction::Parsed(i) => match i {
                UiParsedInstruction::PartiallyDecoded(i) => Some(i),
                _ => None,
            },
            UiInstruction::Compiled(_) => None,
        })
        .filter(|ix| {
            // Only include recognized programs with sufficient accounts for a swap
            RECOGNIZED_POOL_OWNER_PROGRAMS.contains(&ix.program_id) && ix.accounts.len() >= 5
        })
        .map(|ix| (ix.program_id.clone(), ix))
        .collect()
}

pub fn parse_swap_inner_ix(
    ix: &UiPartiallyDecodedInstruction,
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> anyhow::Result<SwapInstruction> {
    use crate::arb::constant::dex_type::DexType;
    use crate::arb::constant::mint::MintPair;
    use crate::arb::pool::interface::InputAccountUtil;
    use crate::arb::pool::meteora_damm_v2::input_account::MeteoraDammV2InputAccount;
    use crate::arb::pool::meteora_dlmm::input_account::MeteoraDlmmInputAccounts;

    match ix.program_id.as_str() {
        PoolOwnerPrograms::METEORA_DLMM => {
            let accounts = MeteoraDlmmInputAccounts::restore_from(ix, tx)?;
            Ok(SwapInstruction {
                dex_type: DexType::MeteoraDlmm,
                pool_address: accounts.lb_pair.pubkey,
                accounts: accounts.to_list().into_iter().cloned().collect(),
                mints: MintPair(accounts.token_x_mint.pubkey, accounts.token_y_mint.pubkey),
            })
        }
        PoolOwnerPrograms::METEORA_DAMM_V2 => {
            let accounts = MeteoraDammV2InputAccount::restore_from(ix, tx)?;
            Ok(SwapInstruction {
                dex_type: DexType::MeteoraDammV2,
                pool_address: accounts.pool.pubkey,
                accounts: accounts.to_list().into_iter().cloned().collect(),
                mints: MintPair(accounts.token_a_mint.pubkey, accounts.token_b_mint.pubkey),
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported program: {}", ix.program_id)),
    }
}