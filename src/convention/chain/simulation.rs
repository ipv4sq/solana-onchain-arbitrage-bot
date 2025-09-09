use crate::global::constant::token_program::TokenProgram;
use crate::util::alias::AResult;
use base64::Engine;
use solana_account_decoder::UiAccountData;
use solana_client::rpc_response::{RpcSimulateTransactionResult, Response};
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Account as TokenAccount;
use spl_token_2022::extension::StateWithExtensions;

#[derive(Debug, Clone)]
pub struct SimulationResponse {
    pub compute_units: Option<u64>,
    pub logs: Vec<String>,
    pub error: Option<String>,
    pub accounts: Vec<SimulatedAccount>,
    pub return_data: Option<ReturnData>,
}

#[derive(Debug, Clone)]
pub struct SimulatedAccount {
    pub pubkey: Pubkey,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
}

#[derive(Debug, Clone)]
pub struct ReturnData {
    pub program_id: Pubkey,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub decimals: u8,
}

impl SimulationResponse {
    pub fn from_rpc_response(
        response: Response<RpcSimulateTransactionResult>,
        account_addresses: &[Pubkey],
    ) -> AResult<Self> {
        let value = response.value;
        
        let mut accounts = Vec::new();
        if let Some(rpc_accounts) = value.accounts {
            for (i, account_opt) in rpc_accounts.into_iter().enumerate() {
                if let Some(account) = account_opt {
                    let pubkey = if i < account_addresses.len() {
                        account_addresses[i]
                    } else {
                        continue;
                    };
                    
                    let data = match &account.data {
                        UiAccountData::Binary(base64_str, _) => {
                            base64::engine::general_purpose::STANDARD.decode(base64_str)?
                        }
                        _ => vec![],
                    };
                    
                    accounts.push(SimulatedAccount {
                        pubkey,
                        lamports: account.lamports,
                        data,
                        owner: account.owner.parse()?,
                        executable: account.executable,
                        rent_epoch: account.rent_epoch,
                    });
                }
            }
        }
        
        let return_data = value.return_data.map(|rd| {
            let data = base64::engine::general_purpose::STANDARD
                .decode(&rd.data.0)
                .unwrap_or_default();
            ReturnData {
                program_id: rd.program_id.parse().unwrap_or_default(),
                data,
            }
        });
        
        Ok(SimulationResponse {
            compute_units: value.units_consumed,
            logs: value.logs.unwrap_or_default(),
            error: value.err.map(|e| format!("{:?}", e)),
            accounts,
            return_data,
        })
    }
    
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
    
    pub fn get_account(&self, pubkey: &Pubkey) -> Option<&SimulatedAccount> {
        self.accounts.iter().find(|a| &a.pubkey == pubkey)
    }
    
    pub fn get_token_balance(&self, pubkey: &Pubkey) -> AResult<Option<TokenBalance>> {
        if let Some(account) = self.get_account(pubkey) {
            if account.owner == TokenProgram::SPL_TOKEN && account.data.len() >= TokenAccount::LEN {
                let token_account = TokenAccount::unpack(&account.data)?;
                Ok(Some(TokenBalance {
                    mint: token_account.mint,
                    owner: token_account.owner,
                    amount: token_account.amount,
                    decimals: 0, // Would need to fetch from mint
                }))
            } else if account.owner == TokenProgram::TOKEN_2022 && account.data.len() >= TokenAccount::LEN {
                let state = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data)?;
                Ok(Some(TokenBalance {
                    mint: state.base.mint,
                    owner: state.base.owner,
                    amount: state.base.amount,
                    decimals: 0, // Would need to fetch from mint
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    pub fn get_token_balances(&self) -> AResult<Vec<TokenBalance>> {
        let mut balances = Vec::new();
        for account in &self.accounts {
            if account.owner == TokenProgram::SPL_TOKEN && account.data.len() >= TokenAccount::LEN {
                if let Ok(token_account) = TokenAccount::unpack(&account.data) {
                    balances.push(TokenBalance {
                        mint: token_account.mint,
                        owner: token_account.owner,
                        amount: token_account.amount,
                        decimals: 0, // Would need to fetch from mint
                    });
                }
            } else if account.owner == TokenProgram::TOKEN_2022 && account.data.len() >= TokenAccount::LEN {
                if let Ok(state) = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account.data) {
                    balances.push(TokenBalance {
                        mint: state.base.mint,
                        owner: state.base.owner,
                        amount: state.base.amount,
                        decimals: 0, // Would need to fetch from mint
                    });
                }
            }
        }
        Ok(balances)
    }
    
    pub fn compare_token_balances(
        &self,
        before: &SimulationResponse,
        token_in: &Pubkey,
        token_out: &Pubkey,
    ) -> AResult<(i128, i128)> {
        let before_in = before
            .get_token_balance(token_in)?
            .map(|b| b.amount)
            .unwrap_or(0);
        let before_out = before
            .get_token_balance(token_out)?
            .map(|b| b.amount)
            .unwrap_or(0);
            
        let after_in = self
            .get_token_balance(token_in)?
            .map(|b| b.amount)
            .unwrap_or(0);
        let after_out = self
            .get_token_balance(token_out)?
            .map(|b| b.amount)
            .unwrap_or(0);
            
        Ok((
            after_in as i128 - before_in as i128,
            after_out as i128 - before_out as i128,
        ))
    }
}

impl SimulatedAccount {
    pub fn as_token_account(&self) -> AResult<Option<TokenAccount>> {
        if self.owner == TokenProgram::SPL_TOKEN && self.data.len() >= TokenAccount::LEN {
            Ok(Some(TokenAccount::unpack(&self.data)?))
        } else if self.owner == TokenProgram::TOKEN_2022 && self.data.len() >= TokenAccount::LEN {
            let state = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&self.data)?;
            Ok(Some(TokenAccount {
                mint: state.base.mint,
                owner: state.base.owner,
                amount: state.base.amount,
                delegate: state.base.delegate.into(),
                state: match state.base.state {
                    spl_token_2022::state::AccountState::Uninitialized => spl_token::state::AccountState::Uninitialized,
                    spl_token_2022::state::AccountState::Initialized => spl_token::state::AccountState::Initialized,
                    spl_token_2022::state::AccountState::Frozen => spl_token::state::AccountState::Frozen,
                },
                is_native: state.base.is_native.into(),
                delegated_amount: state.base.delegated_amount,
                close_authority: state.base.close_authority.into(),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn get_token_balance(&self) -> AResult<Option<u64>> {
        if self.owner == TokenProgram::SPL_TOKEN && self.data.len() >= TokenAccount::LEN {
            Ok(Some(TokenAccount::unpack(&self.data)?.amount))
        } else if self.owner == TokenProgram::TOKEN_2022 && self.data.len() >= TokenAccount::LEN {
            Ok(Some(
                StateWithExtensions::<spl_token_2022::state::Account>::unpack(&self.data)?
                    .base
                    .amount,
            ))
        } else {
            Ok(None)
        }
    }
}

pub struct SimulationHelper;

impl SimulationHelper {
    pub fn format_amount(amount: u64, decimals: u32) -> String {
        let divisor = 10u64.pow(decimals);
        let whole = amount / divisor;
        let fraction = amount % divisor;
        format!("{}.{:0width$}", whole, fraction, width = decimals as usize)
    }
    
    pub fn format_amount_with_raw(amount: u64, decimals: u32) -> String {
        format!("{} ({})", Self::format_amount(amount, decimals), amount)
    }
}