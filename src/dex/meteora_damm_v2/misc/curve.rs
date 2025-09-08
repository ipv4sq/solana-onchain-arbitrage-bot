use anyhow::Result;
use ruint::aliases::U256;

const RESOLUTION: u8 = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rounding {
    Up,
    Down,
}

pub fn mul_div_u256(a: U256, b: U256, denominator: U256, round: Rounding) -> Option<U256> {
    if denominator == U256::ZERO {
        return None;
    }

    let product = a.checked_mul(b)?;

    match round {
        Rounding::Up => Some(product.div_ceil(denominator)),
        Rounding::Down => Some(product / denominator),
    }
}

pub fn get_delta_amount_a_unsigned(
    lower_sqrt_price: u128,
    upper_sqrt_price: u128,
    liquidity: u128,
    round: Rounding,
) -> Result<u64> {
    let result = get_delta_amount_a_unsigned_unchecked(
        lower_sqrt_price,
        upper_sqrt_price,
        liquidity,
        round,
    )?;
    
    if result > U256::from(u64::MAX) {
        return Err(anyhow::anyhow!("Math overflow"));
    }
    
    Ok(result.to::<u64>())
}

pub fn get_delta_amount_a_unsigned_unchecked(
    lower_sqrt_price: u128,
    upper_sqrt_price: u128,
    liquidity: u128,
    round: Rounding,
) -> Result<U256> {
    let numerator_1 = U256::from(liquidity);
    let numerator_2 = U256::from(upper_sqrt_price - lower_sqrt_price);

    let denominator = U256::from(lower_sqrt_price)
        .checked_mul(U256::from(upper_sqrt_price))
        .ok_or_else(|| anyhow::anyhow!("Math overflow"))?;

    if denominator == U256::ZERO {
        return Err(anyhow::anyhow!("Division by zero"));
    }

    mul_div_u256(numerator_1, numerator_2, denominator, round)
        .ok_or_else(|| anyhow::anyhow!("Math overflow"))
}

pub fn get_delta_amount_b_unsigned(
    lower_sqrt_price: u128,
    upper_sqrt_price: u128,
    liquidity: u128,
    round: Rounding,
) -> Result<u64> {
    let result = get_delta_amount_b_unsigned_unchecked(
        lower_sqrt_price,
        upper_sqrt_price,
        liquidity,
        round,
    )?;
    
    if result > U256::from(u64::MAX) {
        return Err(anyhow::anyhow!("Math overflow"));
    }
    
    Ok(result.to::<u64>())
}

pub fn get_delta_amount_b_unsigned_unchecked(
    lower_sqrt_price: u128,
    upper_sqrt_price: u128,
    liquidity: u128,
    round: Rounding,
) -> Result<U256> {
    let liquidity = U256::from(liquidity);
    let delta_sqrt_price = U256::from(upper_sqrt_price - lower_sqrt_price);
    let prod = liquidity
        .checked_mul(delta_sqrt_price)
        .ok_or_else(|| anyhow::anyhow!("Math overflow"))?;

    match round {
        Rounding::Up => {
            let denominator = U256::from(1) << ((RESOLUTION as usize) * 2);
            Ok(prod.div_ceil(denominator))
        }
        Rounding::Down => {
            Ok(prod >> ((RESOLUTION as usize) * 2))
        }
    }
}

pub fn get_next_sqrt_price_from_input(
    sqrt_price: u128,
    liquidity: u128,
    amount_in: u64,
    a_for_b: bool,
) -> Result<u128> {
    if sqrt_price == 0 || liquidity == 0 {
        return Err(anyhow::anyhow!("Invalid sqrt_price or liquidity"));
    }

    if a_for_b {
        get_next_sqrt_price_from_amount_a_rounding_up(sqrt_price, liquidity, amount_in)
    } else {
        get_next_sqrt_price_from_amount_b_rounding_down(sqrt_price, liquidity, amount_in)
    }
}

pub fn get_next_sqrt_price_from_amount_a_rounding_up(
    sqrt_price: u128,
    liquidity: u128,
    amount: u64,
) -> Result<u128> {
    if amount == 0 {
        return Ok(sqrt_price);
    }
    
    let sqrt_price_u256 = U256::from(sqrt_price);
    let liquidity_u256 = U256::from(liquidity);

    let product = U256::from(amount)
        .checked_mul(sqrt_price_u256)
        .ok_or_else(|| anyhow::anyhow!("Math overflow"))?;
    
    let denominator = liquidity_u256
        .checked_add(product)
        .ok_or_else(|| anyhow::anyhow!("Math overflow"))?;
    
    let result = mul_div_u256(liquidity_u256, sqrt_price_u256, denominator, Rounding::Up)
        .ok_or_else(|| anyhow::anyhow!("Math overflow"))?;
    
    if result > U256::from(u128::MAX) {
        return Err(anyhow::anyhow!("Result exceeds u128"));
    }
    
    Ok(result.to::<u128>())
}

pub fn get_next_sqrt_price_from_amount_b_rounding_down(
    sqrt_price: u128,
    liquidity: u128,
    amount: u64,
) -> Result<u128> {
    let quotient = (U256::from(amount) << ((RESOLUTION * 2) as usize)) / U256::from(liquidity);
    
    let result = U256::from(sqrt_price)
        .checked_add(quotient)
        .ok_or_else(|| anyhow::anyhow!("Math overflow"))?;
    
    if result > U256::from(u128::MAX) {
        return Err(anyhow::anyhow!("Result exceeds u128"));
    }
    
    Ok(result.to::<u128>())
}