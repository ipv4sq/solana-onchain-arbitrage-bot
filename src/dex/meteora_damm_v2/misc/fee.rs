use super::pool_data_type::{BaseFeeStruct, DynamicFeeStruct, PoolFeesStruct};
use super::curve::Rounding;
use anyhow::Result;
use ruint::aliases::U256;

const FEE_DENOMINATOR: u64 = 1_000_000_000;
const MAX_FEE_NUMERATOR: u64 = 100_000_000;
const BASIS_POINT_MAX: u64 = 10_000;
const ONE_Q64: u128 = 1 << 64;

#[derive(Debug, PartialEq)]
pub struct FeeOnAmountResult {
    pub amount: u64,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct FeeMode {
    pub fees_on_input: bool,
    pub fees_on_token_a: bool,
    pub has_referral: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FeeSchedulerMode {
    Linear = 0,
    Exponential = 1,
}

impl TryFrom<u8> for FeeSchedulerMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(FeeSchedulerMode::Linear),
            1 => Ok(FeeSchedulerMode::Exponential),
            _ => Err(anyhow::anyhow!("Invalid fee scheduler mode")),
        }
    }
}

pub fn safe_mul_div_cast_u64(
    amount: u64,
    numerator: u64,
    denominator: u64,
    round: Rounding,
) -> Result<u64> {
    if denominator == 0 {
        return Err(anyhow::anyhow!("Division by zero"));
    }
    
    let product = (amount as u128) * (numerator as u128);
    let result = match round {
        Rounding::Up => {
            (product + (denominator as u128 - 1)) / (denominator as u128)
        }
        Rounding::Down => {
            product / (denominator as u128)
        }
    };
    
    if result > u64::MAX as u128 {
        return Err(anyhow::anyhow!("Math overflow"));
    }
    
    Ok(result as u64)
}

pub fn safe_shl_div_cast(
    numerator: u128,
    denominator: u128,
    shift: u8,
    round: Rounding,
) -> Result<u128> {
    if denominator == 0 {
        return Err(anyhow::anyhow!("Division by zero"));
    }
    
    let numerator_u256 = U256::from(numerator) << (shift as usize);
    let denominator_u256 = U256::from(denominator);
    
    let result = match round {
        Rounding::Up => numerator_u256.div_ceil(denominator_u256),
        Rounding::Down => numerator_u256 / denominator_u256,
    };
    
    if result > U256::from(u128::MAX) {
        return Err(anyhow::anyhow!("Math overflow"));
    }
    
    Ok(result.to::<u128>())
}

impl BaseFeeStruct {
    pub fn get_current_base_fee_numerator(
        &self,
        current_point: u64,
        activation_point: u64,
    ) -> Result<u64> {
        if self.period_frequency == 0 {
            return Ok(self.cliff_fee_numerator);
        }
        
        let period = if current_point < activation_point {
            self.number_of_period as u64
        } else {
            let period = (current_point - activation_point) / self.period_frequency;
            period.min(self.number_of_period as u64)
        };
        
        let fee_scheduler_mode = FeeSchedulerMode::try_from(self.fee_scheduler_mode)?;
        
        match fee_scheduler_mode {
            FeeSchedulerMode::Linear => {
                let reduction = period * self.reduction_factor;
                if reduction > self.cliff_fee_numerator {
                    Ok(0)
                } else {
                    Ok(self.cliff_fee_numerator - reduction)
                }
            }
            FeeSchedulerMode::Exponential => {
                let period = period.min(u16::MAX as u64) as u16;
                get_fee_in_period(self.cliff_fee_numerator, self.reduction_factor, period)
            }
        }
    }
}

fn get_fee_in_period(initial_fee: u64, reduction_factor: u64, period: u16) -> Result<u64> {
    let mut fee = initial_fee;
    for _ in 0..period {
        fee = (fee * (BASIS_POINT_MAX - reduction_factor)) / BASIS_POINT_MAX;
    }
    Ok(fee)
}

impl DynamicFeeStruct {
    pub fn is_dynamic_fee_enable(&self) -> bool {
        self.initialized != 0
    }

    pub fn get_variable_fee(&self) -> Result<u128> {
        if !self.is_dynamic_fee_enable() {
            return Ok(0);
        }
        
        let variable_fee = (self.volatility_accumulator as u128 * self.variable_fee_control as u128) 
            / BASIS_POINT_MAX as u128;
        Ok(variable_fee)
    }

    pub fn get_delta_bin_id(
        bin_step_u128: u128,
        sqrt_price_a: u128,
        sqrt_price_b: u128,
    ) -> Result<u128> {
        let (upper_sqrt_price, lower_sqrt_price) = if sqrt_price_a > sqrt_price_b {
            (sqrt_price_a, sqrt_price_b)
        } else {
            (sqrt_price_b, sqrt_price_a)
        };

        let price_ratio = safe_shl_div_cast(upper_sqrt_price, lower_sqrt_price, 64, Rounding::Down)?;
        
        if price_ratio < ONE_Q64 {
            return Ok(0);
        }
        
        let delta_bin_id = (price_ratio - ONE_Q64) / bin_step_u128;
        Ok(delta_bin_id * 2)
    }

    pub fn update_volatility_accumulator(&mut self, sqrt_price: u128) -> Result<()> {
        let delta_price = Self::get_delta_bin_id(self.bin_step_u128, sqrt_price, self.sqrt_price_reference)?;
        
        let volatility_accumulator = (self.volatility_reference as u128) + (delta_price * BASIS_POINT_MAX as u128);
        
        self.volatility_accumulator = volatility_accumulator.min(self.max_volatility_accumulator as u128) as u64;
        Ok(())
    }

    pub fn update_references(
        &mut self,
        sqrt_price_current: u128,
        current_timestamp: u64,
    ) -> Result<()> {
        let elapsed = current_timestamp.saturating_sub(self.last_update_timestamp);
        
        if elapsed >= self.filter_period as u64 {
            self.sqrt_price_reference = sqrt_price_current;
            
            if elapsed < self.decay_period as u64 {
                let volatility_reference = (self.volatility_accumulator as u128 * self.reduction_factor as u128) 
                    / BASIS_POINT_MAX as u128;
                self.volatility_reference = volatility_reference as u64;
            } else {
                self.volatility_reference = 0;
            }
        }
        Ok(())
    }
}

impl PoolFeesStruct {
    pub fn get_total_trading_fee(&self, current_point: u64, activation_point: u64) -> Result<u128> {
        let base_fee_numerator = self.base_fee
            .get_current_base_fee_numerator(current_point, activation_point)?;
        
        let variable_fee = self.dynamic_fee.get_variable_fee()?;
        
        let total_fee_numerator = (base_fee_numerator as u128) + variable_fee;
        Ok(total_fee_numerator)
    }

    pub fn get_fee_on_amount(
        &self,
        amount: u64,
        has_referral: bool,
        current_point: u64,
        activation_point: u64,
        has_partner: bool,
    ) -> Result<FeeOnAmountResult> {
        let trade_fee_numerator = self.get_total_trading_fee(current_point, activation_point)?;
        let trade_fee_numerator = if trade_fee_numerator > MAX_FEE_NUMERATOR as u128 {
            MAX_FEE_NUMERATOR
        } else {
            trade_fee_numerator as u64
        };
        
        let lp_fee = safe_mul_div_cast_u64(amount, trade_fee_numerator, FEE_DENOMINATOR, Rounding::Up)?;
        
        let amount = amount.saturating_sub(lp_fee);
        
        let protocol_fee = safe_mul_div_cast_u64(
            lp_fee,
            self.protocol_fee_percent as u64,
            100,
            Rounding::Down,
        )?;
        
        let lp_fee = lp_fee.saturating_sub(protocol_fee);
        
        let referral_fee = if has_referral {
            safe_mul_div_cast_u64(
                protocol_fee,
                self.referral_fee_percent as u64,
                100,
                Rounding::Down,
            )?
        } else {
            0
        };
        
        let protocol_fee_after_referral = protocol_fee.saturating_sub(referral_fee);
        
        let partner_fee = if has_partner && self.partner_fee_percent > 0 {
            safe_mul_div_cast_u64(
                protocol_fee_after_referral,
                self.partner_fee_percent as u64,
                100,
                Rounding::Down,
            )?
        } else {
            0
        };
        
        let protocol_fee = protocol_fee_after_referral.saturating_sub(partner_fee);
        
        Ok(FeeOnAmountResult {
            amount,
            lp_fee,
            protocol_fee,
            partner_fee,
            referral_fee,
        })
    }
}