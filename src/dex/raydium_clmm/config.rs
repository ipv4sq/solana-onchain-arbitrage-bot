use crate::convention::chain::instruction::Instruction;
use crate::dex::interface::{PoolBase, PoolConfig, PoolDataLoader};
use crate::dex::raydium_clmm::ix_account::RaydiumClmmIxAccount;
use crate::dex::raydium_clmm::pool_data::RaydiumClmmPoolData;
use crate::global::enums::dex_type::DexType;
use crate::global::enums::dex_type::DexType::RaydiumClmm;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type RaydiumClmmConfig = PoolBase<RaydiumClmmPoolData>;

impl PoolConfig<RaydiumClmmPoolData> for RaydiumClmmConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = RaydiumClmmPoolData::load_data(data)?;
        Ok(RaydiumClmmConfig {
            pool_address: address,
            base_mint: pool_data.token_mint_0,
            quote_mint: pool_data.token_mint_1,
            base_reserve: pool_data.token_vault_0,
            quote_reserve: pool_data.token_vault_1,
            dex_type,
            pool_data,
        })
    }

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&RaydiumClmm.owner_program_id())?;
        let address = ix.account_at(2)?.pubkey;
        Ok((RaydiumClmm, address))
    }

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        let built = RaydiumClmmIxAccount::build_accounts_with_direction(
            payer,
            &self.pool_address,
            &self.pool_data,
            &self.base_mint,
            &self.quote_mint,
        )
        .await?;
        let mut accounts: Vec<AccountMeta> = vec![
            RaydiumClmm.owner_program_id().to_program(),
            self.pool_data.mint_pair().desired_mint()?.to_readonly(),
            self.pool_address.to_writable(),
            built.amm_config,
            built.observation_state,
            self.pool_data.token_vault_0.to_writable(),
            self.pool_data.token_vault_1.to_writable(),
        ];
        accounts.extend(built.tick_arrays);

        Ok(accounts)
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

impl AsRef<PoolBase<RaydiumClmmPoolData>> for RaydiumClmmConfig {
    fn as_ref(&self) -> &PoolBase<RaydiumClmmPoolData> {
        self
    }
}
