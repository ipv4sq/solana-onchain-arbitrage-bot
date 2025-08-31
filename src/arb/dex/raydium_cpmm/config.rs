use crate::arb::convention::chain::instruction::Instruction;
use crate::arb::dex::interface::{PoolBase, PoolConfig, PoolDataLoader};
use crate::arb::dex::meteora_dlmm::price_calculator::DlmmQuote;
use crate::arb::dex::raydium_cpmm::pool_data::RaydiumCpmmAPoolData;
use crate::arb::dex::raydium_cpmm::RAYDIUM_CPMM_AUTHORITY;
use crate::arb::global::constant::mint::Mints;
use crate::arb::global::enums::dex_type::DexType;
use crate::arb::util::alias::{AResult, MintAddress, PoolAddress};
use crate::arb::util::structs::mint_pair::MintPair;
use crate::arb::util::traits::option::OptionExt;
use crate::{bail_error, return_error};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::cmp::min;

pub type RaydiumCpmmConfig = PoolBase<RaydiumCpmmAPoolData>;

impl PoolConfig<RaydiumCpmmAPoolData> for RaydiumCpmmConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = RaydiumCpmmAPoolData::load_data(data)?;
        Ok(RaydiumCpmmConfig {
            pool_address: address,
            base_mint: pool_data.token_0_mint,
            quote_mint: pool_data.token_1_mint,
            dex_type,
            pool_data,
        })
    }

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&DexType::RaydiumCpmm.owner_program_id())?;
        /*
        Copied from https://solscan.io/tx/3RmFU3LLNQnDFqNFMmuTZvUsxP1c4ZttE6QqTvuGFToBdq7PY3tRuBxKkyqs3vCSAs3gKDuGiVi7jtXrpb4UtDpf
        #1 - Payer:
        #2 - Authority:
        #3 - Amm Config:
        #4 - Pool State:
        #11 - Input Token:
        #12 - Output Token:
        */
        if ix.accounts.len() < 12 {
            return_error!(
                "Insufficient accounts for raydium ix: {}",
                ix.accounts.len()
            );
        }
        let account_2 = ix.accounts.get(1).or_err("")?.pubkey;
        let account_4 = ix.accounts.get(3).or_err("")?.pubkey;
        let account_11 = ix.accounts.get(10).or_err("")?.pubkey;
        let account_12 = ix.accounts.get(11).or_err("")?.pubkey;

        if account_2 != RAYDIUM_CPMM_AUTHORITY {
            return_error!("Not authorized to use the raydium cpmm: {}", account_2);
        }

        let mint_pair = MintPair(account_11, account_12);
        if !mint_pair.contains(&Mints::USDC) && !mint_pair.contains(&Mints::WSOL) {
            return_error!("{} doesn't seemt to have usdc or wsol in pair", account_4)
        }

        Ok((DexType::RaydiumCpmm, account_4))
    }

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        todo!()
    }

    async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote> {
        todo!()
    }
}

impl AsRef<PoolBase<RaydiumCpmmAPoolData>> for RaydiumCpmmConfig {
    fn as_ref(&self) -> &PoolBase<RaydiumCpmmAPoolData> {
        self
    }
}
