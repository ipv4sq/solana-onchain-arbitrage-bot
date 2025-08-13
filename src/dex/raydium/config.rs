use crate::constants::addresses::TokenMint;
use crate::constants::helpers::ToPubkey;
use crate::constants::utils::expect_owner;
use crate::dex::pool_checker::PoolChecker;
use crate::dex::pool_fetch::PoolFetch;
use crate::dex::raydium::{
    RaydiumAmmInfo, RaydiumClmmPoolInfo, RaydiumCpAmmInfo, RAYDIUM_CLMM_PROGRAM_ID,
    RAYDIUM_CP_PROGRAM_ID, RAYDIUM_PROGRAM_ID,
};
use crate::not_in;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;

// 老式 AMM（恒定乘积公式，池简单，单 vault 对单 mint）
#[derive(Debug, Clone)]
pub struct RaydiumPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

impl PoolFetch for RaydiumPool {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> anyhow::Result<Self> {
        let account = rpc_client
            .get_account(pool)
            .map_err(|e| anyhow::anyhow!("Error fetching RaydiumPool account {pool}: {e:?}"))?;

        expect_owner(pool, &account, &RAYDIUM_PROGRAM_ID.to_pubkey())?;

        let amm_info = RaydiumAmmInfo::load_checked(&account.data)?;

        if not_in!(*mint, amm_info.coin_mint, amm_info.pc_mint) {
            return Err(anyhow::anyhow!(
                "Invalid Raydium pool: {} because the pair doesn't contain the mint {}",
                pool,
                mint
            ));
        }

        let sol = TokenMint::SOL.to_pubkey();

        if not_in!(sol, amm_info.coin_mint, amm_info.pc_mint) {
            return Err(anyhow::anyhow!(
                "SOL is not present in Raydium pool: {}",
                pool
            ));
        }

        let (sol_vault, token_vault) = if sol == amm_info.coin_mint {
            (amm_info.coin_vault, amm_info.pc_vault)
        } else {
            (amm_info.pc_vault, amm_info.coin_vault)
        };

        let (token_mint, base_mint) = if mint == &amm_info.coin_mint {
            (amm_info.coin_mint, amm_info.pc_mint)
        } else {
            (amm_info.pc_mint, amm_info.coin_mint)
        };
        Ok(RaydiumPool {
            pool: *pool,
            token_vault,
            sol_vault,
            base_mint,
            token_mint,
        })
    }
}

// 集中流动性池（有 tick、position NFT 等复杂结构）
#[derive(Debug, Clone)]
pub struct RaydiumCpPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub amm_config: Pubkey,
    pub observation: Pubkey,
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

impl PoolFetch for RaydiumCpPool {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> anyhow::Result<Self> {
        let account = rpc_client.get_account(pool)?;

        expect_owner(pool, &account, &RAYDIUM_CP_PROGRAM_ID.to_pubkey())?;

        let info = RaydiumCpAmmInfo::load_checked(&account.data)?;

        if not_in!(*mint, info.token_0_mint, info.token_1_mint) {
            return Err(anyhow::anyhow!(
                "Invalid Raydium CP pool: {} missing mint {}",
                pool,
                mint
            ));
        }

        let sol = TokenMint::SOL.to_pubkey();
        if not_in!(sol, info.token_0_mint, info.token_1_mint) {
            return Err(anyhow::anyhow!(
                "SOL is not present in Raydium CP pool: {}",
                pool
            ));
        }

        let (sol_vault, token_vault) = if sol == info.token_0_mint {
            (info.token_0_vault, info.token_1_mint)
        } else {
            (info.token_1_mint, info.token_0_vault)
        };

        let (token_mint, base_mint) = if mint == &info.token_0_mint {
            (info.token_0_mint, info.token_1_mint)
        } else {
            (info.token_1_mint, info.token_0_mint)
        };

        Ok(RaydiumCpPool {
            pool: *pool,
            token_vault,
            sol_vault,
            amm_config: info.amm_config,
            observation: info.observation_key,
            token_mint,
            base_mint,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RaydiumClmmPool {
    pub pool: Pubkey,
    pub amm_config: Pubkey,
    pub observation_state: Pubkey,
    pub bitmap_extension: Pubkey,
    pub x_vault: Pubkey,
    pub y_vault: Pubkey,
    pub tick_arrays: Vec<Pubkey>,
    pub memo_program: Option<Pubkey>, // For Token 2022 support
    pub token_mint: Pubkey,
    pub base_mint: Pubkey,
}

impl PoolFetch for RaydiumClmmPool {
    fn fetch(pool: &Pubkey, mint: &Pubkey, rpc_client: &RpcClient) -> anyhow::Result<Self> {
        let account = rpc_client.get_account(pool)?;
        expect_owner(pool, &account, &RAYDIUM_CLMM_PROGRAM_ID.to_pubkey())?;

        let info = RaydiumClmmPoolInfo::load_checked(&account.data)?;
        let sol = TokenMint::SOL.to_pubkey();
        info.consists_of(mint, &sol, Some(pool))?;

        Ok(RaydiumClmmPool {
            pool: *pool,
            amm_config: info.amm_config,
            observation_state: info.observation_key,
            bitmap_extension: info.get_bitmap_extensions(pool),
            // I think there might be some bug in the original implementation
            x_vault: info.get_base_vault(),
            y_vault: info.get_token_vault(),
            tick_arrays: info.get_tick_arrays(pool)?,
            memo_program: None,
            token_mint: info.get_not_sol_mint()?,
            base_mint: info.get_sol_mint()?,
        })
    }
}
