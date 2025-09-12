use crate::convention::chain::instruction::Instruction;
use crate::dex::interface::{PoolBase, PoolConfig, PoolDataLoader};
use crate::dex::legacy_interface::InputAccountUtil;
use crate::dex::meteora_damm_v2::misc::input_account::MeteoraDammV2InputAccount;
use crate::dex::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use crate::dex::EstimatedQuote;
use crate::global::enums::dex_type::DexType;
use crate::global::enums::dex_type::DexType::MeteoraDammV2;
use crate::util::alias::{AResult, MintAddress, PoolAddress};
use crate::util::traits::account_meta::ToAccountMeta;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

pub type MeteoraDammV2Config = PoolBase<MeteoraDammV2PoolData>;

impl PoolConfig<MeteoraDammV2PoolData> for MeteoraDammV2Config {
    fn from_data(address: PoolAddress, dex_type: DexType, data: &[u8]) -> AResult<Self> {
        let pool_data = MeteoraDammV2PoolData::load_data(data)?;
        Ok(MeteoraDammV2Config {
            pool_address: address,
            base_mint: pool_data.token_a_mint,
            base_reserve: pool_data.token_a_vault,
            quote_mint: pool_data.token_b_mint,
            quote_reserve: pool_data.token_b_vault,
            dex_type,
            pool_data,
        })
    }

    fn pase_swap_from_ix(ix: &Instruction) -> AResult<(DexType, PoolAddress)> {
        ix.expect_program_id(&MeteoraDammV2.owner_program_id())?;
        let address = ix.account_at(1)?.pubkey;
        Ok((MeteoraDammV2, address))
    }

    async fn build_mev_bot_ix_accounts(&self, payer: &Pubkey) -> AResult<Vec<AccountMeta>> {
        let built = MeteoraDammV2InputAccount::build_accounts_no_matter_direction_size(
            payer,
            &self.pool_address,
            &self.pool_data,
        )
        .await?;
        let accounts: Vec<AccountMeta> = vec![
            built.meteora_program,
            self.pool_data.mint_pair().desired_mint()?.to_readonly(),
            built.event_authority,
            built.pool_authority,
            self.pool_address.to_writable(),
            built.token_a_vault,
            built.token_b_vault,
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

impl AsRef<PoolBase<MeteoraDammV2PoolData>> for MeteoraDammV2Config {
    fn as_ref(&self) -> &PoolBase<MeteoraDammV2PoolData> {
        self
    }
}
