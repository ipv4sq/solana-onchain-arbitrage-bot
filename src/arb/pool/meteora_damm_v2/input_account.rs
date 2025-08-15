use crate::arb::pool::interface::SwapInputAccountUtil;
use crate::arb::pool::meteora_damm_v2::pool_data::MeteoraDammV2PoolData;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiPartiallyDecodedInstruction,
};
pub struct MeteoraDammV2InputAccount {
    pub pool_authority: AccountMeta,
    pub pool_market: AccountMeta,
    pub input_token_account: AccountMeta,
    pub output_token_account: AccountMeta,
    pub token_a_vault: AccountMeta,
    pub token_b_vault: AccountMeta,
    pub token_a_mint: AccountMeta,
    pub token_b_mint: AccountMeta,
    pub payer: AccountMeta,
    pub token_a_program: AccountMeta,
    pub token_b_program: AccountMeta,
    // solscan 上显示的是account, 但是实际上的地址是一个program
    // https://solscan.io/tx/57kgd8oiLFRmRyFR5dKwUoTggoP25FyBKsqqGpm58pJ3qAUE8WPhQXECjGjx5ATF87qP7MMjmZK45qACoTB476eP
    pub referral_token_program: AccountMeta,
    pub event_authority: AccountMeta,
    pub meteora_program: AccountMeta,
}

/*
那么对于这个account, 我们需要实现哪些函数呢?
1. 对于一个已经存在的IX, 我们需要能从一堆accounts中, 按照顺序restore出这个数据结构, 当然, 权限信息需要从tx中拿到.
2. 对于一个我们想发起的交易, 我们要能够从pool_data, 交易对, 交易方向, 交易数量, 以及池子本身的地址中推导出需要的accounts. 有些时候甚至需要RPC.
3. 实现一个to_list方法,把结果推成一个list
 */
impl SwapInputAccountUtil<MeteoraDammV2InputAccount, MeteoraDammV2PoolData>
    for MeteoraDammV2InputAccount
{
    fn retore_from(
        ix: &UiPartiallyDecodedInstruction,
        tx: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<MeteoraDammV2InputAccount> {
        todo!()
    }

    fn build_accounts(
        pool: &Pubkey,
        pool_data: MeteoraDammV2PoolData,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: Option<u64>,
        output_amount: Option<u64>,
        rpc: &RpcClient,
    ) -> Result<MeteoraDammV2InputAccount> {
        todo!()
    }

    fn to_list(&self) -> Vec<&AccountMeta> {
        todo!()
    }
}
