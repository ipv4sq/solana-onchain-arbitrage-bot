use crate::arb::convention::chain::util::ownership::expect_owner;
use crate::arb::global::constant::mint::Mints;
use crate::arb::util::traits::pubkey::ToPubkey;
use crate::legacy_dex::meteora::constants::{METEORA_DAMM_PROGRAM_ID, METEORA_DLMM_PROGRAM_ID};
use crate::legacy_dex::meteora::pool_dlmm_info::MeteoraDlmmInfo;
use crate::legacy_dex::pool_checker::PoolChecker;
use crate::legacy_dex::pool_fetch::PoolFetch;
use meteora_damm_cpi::Pool;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct MeteoraDAmmPool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
    pub token_x_token_vault: Pubkey,
    pub token_sol_token_vault: Pubkey,
    pub token_x_lp_mint: Pubkey,
    pub token_sol_lp_mint: Pubkey,
    pub token_x_pool_lp: Pubkey,
    pub token_sol_pool_lp: Pubkey,
    pub admin_token_fee_x: Pubkey,
    pub admin_token_fee_sol: Pubkey,
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}
impl PoolChecker for Pool {
    fn get_base_mint(&self) -> Pubkey {
        self.token_a_mint
    }

    fn get_token_mint(&self) -> Pubkey {
        self.token_b_mint
    }

    fn get_base_vault(&self) -> Pubkey {
        self.a_vault
    }

    fn get_token_vault(&self) -> Pubkey {
        self.b_vault
    }
}

impl PoolFetch for MeteoraDAmmPool {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> anyhow::Result<Self> {
        let account = rpc_client.get_account(pool)?;
        expect_owner(pool, &account, &METEORA_DAMM_PROGRAM_ID.to_pubkey())?;

        let info = meteora_damm_cpi::Pool::deserialize_unchecked(&account.data)?;

        let sol = Mints::WSOL;
        info.consists_of(&sol, mint, Some(pool))?;

        Ok(MeteoraDAmmPool {
            pool: *pool,
            token_x_vault: info.get_base_vault(),
            token_sol_vault: info.get_sol_vault()?,
            token_x_token_vault: Default::default(),
            token_sol_token_vault: Default::default(),
            token_x_lp_mint: Default::default(),
            token_sol_lp_mint: Default::default(),
            token_x_pool_lp: Default::default(),
            token_sol_pool_lp: Default::default(),
            admin_token_fee_x: Default::default(),
            admin_token_fee_sol: Default::default(),
            token_mint: Default::default(),
            base_mint: Default::default(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct MeteoraDAmmV2Pool {
    pub pool: Pubkey,
    pub token_x_vault: Pubkey,
    pub token_sol_vault: Pubkey,
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

#[derive(Debug, Clone)]
pub struct MeteoraDlmmPool {
    pub pair: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub oracle: Pubkey,
    pub bin_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

impl PoolFetch for MeteoraDlmmPool {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> anyhow::Result<Self> {
        let account = rpc_client.get_account(pool)?;
        expect_owner(pool, &account, &METEORA_DLMM_PROGRAM_ID.to_pubkey())?;

        let info = MeteoraDlmmInfo::load_checked(&account.data)?;
        let sol = Mints::WSOL;
        info.consists_of(mint, &sol, Some(pool))?;

        Ok(MeteoraDlmmPool {
            pair: *pool,
            token_vault: info.get_token_vault(),
            sol_vault: info.get_sol_vault()?,
            oracle: info.oracle,
            bin_arrays: info.calculate_bin_arrays(pool)?,
            memo_program: None,
            token_mint: info.get_not_sol_mint()?,
            base_mint: info.get_sol_mint()?,
        })
    }
}
