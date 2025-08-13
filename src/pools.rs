use crate::dex::meteora::config::{MeteoraDAmmPool, MeteoraDAmmV2Pool, MeteoraDlmmPool};
use crate::dex::pump::PumpPool;
use crate::dex::raydium::config::{RaydiumClmmPool, RaydiumCpPool, RaydiumPool};
use crate::dex::solfi::config::SolfiPool;
use crate::dex::vertigo::config::VertigoPool;
use crate::dex::whirlpool::config::WhirlpoolPool;
use crate::{
    constants::addresses::TokenMint,
    dex::raydium::{pool_clmm_info::POOL_TICK_ARRAY_BITMAP_SEED, raydium_clmm_program_id},
};
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
        let sol_mint = Pubkey::from_str(TokenMint::SOL)?;
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


    pub fn add_dlmm_pool(
        &mut self,
        pair: &str,
        token_vault: &str,
        sol_vault: &str,
        oracle: &str,
        bin_arrays: Vec<&str>,
        memo_program: Option<&str>,
        token_mint: &str,
        base_mint: &str,
    ) -> anyhow::Result<()> {
        let bin_array_pubkeys = bin_arrays
            .iter()
            .map(|&s| Pubkey::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;

        let memo_program_pubkey = if let Some(memo) = memo_program {
            Some(Pubkey::from_str(memo)?)
        } else {
            None
        };

        self.meteora_dlmm_pools.push(MeteoraDlmmPool {
            pair: Pubkey::from_str(pair)?,
            token_vault: Pubkey::from_str(token_vault)?,
            sol_vault: Pubkey::from_str(sol_vault)?,
            oracle: Pubkey::from_str(oracle)?,
            bin_arrays: bin_array_pubkeys,
            memo_program: memo_program_pubkey,
            token_mint: Pubkey::from_str(token_mint)?,
            base_mint: Pubkey::from_str(base_mint)?,
        });
        Ok(())
    }

    pub fn add_whirlpool_pool(
        &mut self,
        pool: &str,
        oracle: &str,
        x_vault: &str,
        y_vault: &str,
        tick_arrays: Vec<&str>,
        memo_program: Option<&str>,
        token_mint: &str,
        base_mint: &str,
    ) -> anyhow::Result<()> {
        let tick_array_pubkeys = tick_arrays
            .iter()
            .map(|&s| Pubkey::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;

        let memo_program_pubkey = if let Some(memo) = memo_program {
            Some(Pubkey::from_str(memo)?)
        } else {
            None
        };

        self.whirlpool_pools.push(WhirlpoolPool {
            pool: Pubkey::from_str(pool)?,
            oracle: Pubkey::from_str(oracle)?,
            x_vault: Pubkey::from_str(x_vault)?,
            y_vault: Pubkey::from_str(y_vault)?,
            tick_arrays: tick_array_pubkeys,
            memo_program: memo_program_pubkey,
            token_mint: Pubkey::from_str(token_mint)?,
            base_mint: Pubkey::from_str(base_mint)?,
        });
        Ok(())
    }

    pub fn add_raydium_clmm_pool(
        &mut self,
        pool: &str,
        amm_config: &str,
        observation_state: &str,
        x_vault: &str,
        y_vault: &str,
        tick_arrays: Vec<&str>,
        memo_program: Option<&str>,
        token_mint: &str,
        base_mint: &str,
    ) -> anyhow::Result<()> {
        let pool_pubkey = Pubkey::from_str(pool)?;
        let bitmap_extension = Pubkey::find_program_address(
            &[
                POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                &pool_pubkey.as_ref(),
            ],
            &raydium_clmm_program_id(),
        )
        .0;
        let tick_array_pubkeys = tick_arrays
            .iter()
            .map(|&s| Pubkey::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;

        let memo_program_pubkey = if let Some(memo) = memo_program {
            Some(Pubkey::from_str(memo)?)
        } else {
            None
        };

        self.raydium_clmm_pools.push(RaydiumClmmPool {
            pool: pool_pubkey,
            amm_config: Pubkey::from_str(amm_config)?,
            observation_state: Pubkey::from_str(observation_state)?,
            x_vault: Pubkey::from_str(x_vault)?,
            y_vault: Pubkey::from_str(y_vault)?,
            bitmap_extension,
            tick_arrays: tick_array_pubkeys,
            memo_program: memo_program_pubkey,
            token_mint: Pubkey::from_str(token_mint)?,
            base_mint: Pubkey::from_str(base_mint)?,
        });
        Ok(())
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
