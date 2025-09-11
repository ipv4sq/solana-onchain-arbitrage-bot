use crate::convention::chain::instruction::Instruction;
use crate::dex::interface::{PoolBase, PoolConfig, PoolDataLoader};
use crate::dex::meteora_dlmm::price::price_calculator::DlmmQuote;
use crate::dex::raydium_cpmm::pool_data::RaydiumCpmmPoolData;
use crate::dex::raydium_cpmm::RAYDIUM_CPMM_AUTHORITY;
use crate::global::constant::mint::Mints;
use crate::global::constant::pool_program::PoolProgram;
use crate::global::enums::dex_type::DexType;
use crate::return_error;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::structs::mint_pair::MintPair;
use crate::util::traits::account_meta::ToAccountMeta;
use crate::util::traits::option::OptionExt;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type RaydiumCpmmConfig = PoolBase<RaydiumCpmmPoolData>;

impl PoolConfig<RaydiumCpmmPoolData> for RaydiumCpmmConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = RaydiumCpmmPoolData::load_data(data)?;
        Ok(RaydiumCpmmConfig {
            pool_address: address,
            base_mint: pool_data.token_0_mint,
            base_reserve: pool_data.token_0_vault,
            quote_mint: pool_data.token_1_mint,
            quote_reserve: pool_data.token_1_vault,
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
        let desired_mint = Mints::WSOL;
        self.pool_data.pair().shall_contain(&desired_mint)?;

        let (minor_mint_vault, desired_mint_vault) = if self.base_mint != desired_mint {
            (self.pool_data.base_vault(), self.pool_data.quote_vault())
        } else {
            (self.pool_data.quote_vault(), self.pool_data.base_vault())
        };

        let accounts: Vec<AccountMeta> = vec![
            PoolProgram::RAYDIUM_CPMM.to_program(),
            self.pool_data.pair().desired_mint()?.to_readonly(),
            RAYDIUM_CPMM_AUTHORITY.to_readonly(),
            self.pool_address.to_writable(),
            self.pool_data.amm_config.to_readonly(),
            minor_mint_vault.to_writable(),
            desired_mint_vault.to_writable(),
            self.pool_data.observation_key.to_writable(),
        ];

        Ok(accounts)
    }

    async fn mid_price(&self, from: &MintAddress, to: &MintAddress) -> AResult<DlmmQuote> {
        todo!()
    }

    async fn get_amount_out(
        &self,
        input_amount: u64,
        from_mint: &MintAddress,
        to_mint: &MintAddress,
    ) -> AResult<u64> {
        self.pool_data
            .get_amount_out(input_amount, from_mint, to_mint)
            .await
    }
}

impl AsRef<PoolBase<RaydiumCpmmPoolData>> for RaydiumCpmmConfig {
    fn as_ref(&self) -> &PoolBase<RaydiumCpmmPoolData> {
        self
    }
}
