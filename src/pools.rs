use crate::dex::meteora::config::{MeteoraDAmmPool, MeteoraDAmmV2Pool, MeteoraDlmmPool};
use crate::dex::pump::PumpPool;
use crate::dex::raydium::config::{RaydiumClmmPool, RaydiumCpPool, RaydiumPool};
use crate::dex::solfi::config::SolfiPool;
use crate::dex::vertigo::config::VertigoPool;
use crate::dex::whirlpool::config::WhirlpoolPool;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MintPoolData {
    pub mint: Pubkey,
    pub token_program: Pubkey, // Support for both Token and Token 2022
    // pub wallet_account: Pubkey,
    pub wallet_wsol_account: Pubkey,

    // below are thr pools
    pub raydium_pools: Vec<RaydiumPool>,
    pub raydium_cp_pools: Vec<RaydiumCpPool>,
    pub raydium_clmm_pools: Vec<RaydiumClmmPool>,
    //
    pub pump_pools: Vec<PumpPool>,
    //
    pub meteora_dlmm_pools: Vec<MeteoraDlmmPool>,
    pub meteora_damm_pools: Vec<MeteoraDAmmPool>,
    pub meteora_damm_v2_pools: Vec<MeteoraDAmmV2Pool>,
    //
    pub whirlpool_pools: Vec<WhirlpoolPool>,
    pub solfi_pools: Vec<SolfiPool>,
    pub vertigo_pools: Vec<VertigoPool>,
}

impl MintPoolData {
    pub fn new(mint: &str, wallet_account: &str, token_program: Pubkey) -> anyhow::Result<Self> {
        let sol_mint = crate::arb::global::constant::mint::Mints::WSOL;
        let wallet_pk = Pubkey::from_str(wallet_account)?;
        let wallet_wsol_pk =
            spl_associated_token_account::get_associated_token_address(&wallet_pk, &sol_mint);
        Ok(Self {
            mint: Pubkey::from_str(mint)?,
            token_program,
            // wallet_account: wallet_pk,
            wallet_wsol_account: wallet_wsol_pk,
            raydium_pools: Vec::new(),
            raydium_cp_pools: Vec::new(),
            pump_pools: Vec::new(),
            meteora_dlmm_pools: Vec::new(),
            whirlpool_pools: Vec::new(),
            raydium_clmm_pools: Vec::new(),
            meteora_damm_pools: Vec::new(),
            solfi_pools: Vec::new(),
            meteora_damm_v2_pools: Vec::new(),
            vertigo_pools: Vec::new(),
        })
    }

    pub fn add_meteora_damm_pool(
        &mut self,
        pool: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
        token_x_token_vault: &str,
        token_sol_token_vault: &str,
        token_x_lp_mint: &str,
        token_sol_lp_mint: &str,
        token_x_pool_lp: &str,
        token_sol_pool_lp: &str,
        admin_token_fee_x: &str,
        admin_token_fee_sol: &str,
        token_mint: &str,
        base_mint: &str,
    ) -> anyhow::Result<()> {
        self.meteora_damm_pools.push(MeteoraDAmmPool {
            pool: Pubkey::from_str(pool)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
            token_x_token_vault: Pubkey::from_str(token_x_token_vault)?,
            token_sol_token_vault: Pubkey::from_str(token_sol_token_vault)?,
            token_x_lp_mint: Pubkey::from_str(token_x_lp_mint)?,
            token_sol_lp_mint: Pubkey::from_str(token_sol_lp_mint)?,
            token_x_pool_lp: Pubkey::from_str(token_x_pool_lp)?,
            token_sol_pool_lp: Pubkey::from_str(token_sol_pool_lp)?,
            admin_token_fee_x: Pubkey::from_str(admin_token_fee_x)?,
            admin_token_fee_sol: Pubkey::from_str(admin_token_fee_sol)?,
            token_mint: Pubkey::from_str(token_mint)?,
            base_mint: Pubkey::from_str(base_mint)?,
        });
        Ok(())
    }

    pub fn add_solfi_pool(
        &mut self,
        pool: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
        token_mint: &str,
        base_mint: &str,
    ) -> anyhow::Result<()> {
        self.solfi_pools.push(SolfiPool {
            pool: Pubkey::from_str(pool)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
            token_mint: Pubkey::from_str(token_mint)?,
            base_mint: Pubkey::from_str(base_mint)?,
        });
        Ok(())
    }

    pub fn add_meteora_damm_v2_pool(
        &mut self,
        pool: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
        token_mint: &str,
        base_mint: &str,
    ) -> anyhow::Result<()> {
        self.meteora_damm_v2_pools.push(MeteoraDAmmV2Pool {
            pool: Pubkey::from_str(pool)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
            token_mint: Pubkey::from_str(token_mint)?,
            base_mint: Pubkey::from_str(base_mint)?,
        });
        Ok(())
    }

    pub fn add_vertigo_pool(
        &mut self,
        pool: &str,
        pool_owner: &str,
        token_x_vault: &str,
        token_sol_vault: &str,
        token_mint: &str,
        base_mint: &str,
    ) -> anyhow::Result<()> {
        self.vertigo_pools.push(VertigoPool {
            pool: Pubkey::from_str(pool)?,
            pool_owner: Pubkey::from_str(pool_owner)?,
            token_x_vault: Pubkey::from_str(token_x_vault)?,
            token_sol_vault: Pubkey::from_str(token_sol_vault)?,
            token_mint: Pubkey::from_str(token_mint)?,
            base_mint: Pubkey::from_str(base_mint)?,
        });
        Ok(())
    }
}
