use crate::arb::convention::chain::util::ownership::expect_owner;
use crate::arb::global::constant::mint::Mints;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::dex::pool_checker::PoolChecker;
use crate::dex::pool_fetch::PoolFetch;
use crate::dex::whirlpool::constants::WHIRLPOOL_PROGRAM_ID;
use crate::dex::whirlpool::get_tick_arrays;
use crate::dex::whirlpool::pool_clmm::WhirlpoolInfo;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct WhirlpoolPool {
    pub pool: Pubkey,
    pub oracle: Pubkey,
    pub x_vault: Pubkey,
    pub y_vault: Pubkey,
    pub tick_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

impl PoolFetch for WhirlpoolPool {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> anyhow::Result<Self> {
        let account = rpc_client.get_account(&pool)?;
        expect_owner(pool, &account, &WHIRLPOOL_PROGRAM_ID.to_pubkey())?;

        let info = WhirlpoolInfo::try_deserialize(&account.data)?;
        let sol = Mints::WSOL;
        info.consists_of(mint, &sol, Some(pool))?;

        Ok(WhirlpoolPool {
            pool: *pool,
            oracle: WhirlpoolInfo::get_oracle(pool),
            x_vault: info.get_base_vault(),
            y_vault: info.get_token_vault(),
            tick_arrays: get_tick_arrays(&info, pool, &WHIRLPOOL_PROGRAM_ID.to_pubkey())
                .iter()
                .map(|x| x.pubkey)
                .collect(),
            memo_program: None,
            token_mint: info.get_not_sol_mint()?,
            base_mint: info.get_sol_mint()?,
        })
    }
}
