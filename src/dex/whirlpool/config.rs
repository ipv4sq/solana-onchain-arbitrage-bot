use crate::convention::chain::instruction::Instruction;
use crate::dex::interface::{PoolBase, PoolConfig, PoolDataLoader};
use crate::dex::whirlpool::ix_account::WhirlpoolIxAccount;
use crate::dex::whirlpool::pool_data::WhirlpoolPoolData;
use crate::global::constant::pool_program::PoolProgram;
use crate::global::enums::dex_type::DexType;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use DexType::Whirlpool;

pub type WhirlpoolConfig = PoolBase<WhirlpoolPoolData>;

impl PoolConfig<WhirlpoolPoolData> for WhirlpoolConfig {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = WhirlpoolPoolData::load_data(data)?;
        Ok(WhirlpoolConfig {
            pool_address: address,
            base_mint: pool_data.token_mint_a,
            quote_mint: pool_data.token_mint_b,
            base_reserve: pool_data.token_vault_a,
            quote_reserve: pool_data.token_vault_b,
            dex_type,
            pool_data,
        })
    }

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&Whirlpool.owner_program_id())?;
        let address = ix.account_at(4)?.pubkey;
        Ok((Whirlpool, address))
    }

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        let built =
            WhirlpoolIxAccount::build_bidirectional(payer, &self.pool_address, &self.pool_data)
                .await?;
        let accounts: Vec<AccountMeta> = vec![
            PoolProgram::WHIRLPOOL.to_program(),
            self.pool_data.mint_pair().desired_mint()?.to_readonly(),
            built.memo_program,
            built.whirlpool,
            built.oracle,
            built.token_vault_a,
            built.token_vault_b,
            built.tick_array_0,
            built.tick_array_1,
            built.tick_array_2,
        ];

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
impl AsRef<PoolBase<WhirlpoolPoolData>> for WhirlpoolConfig {
    fn as_ref(&self) -> &PoolBase<WhirlpoolPoolData> {
        self
    }
}
