use alloy_primitives::{I256, U256};

/// Computes the amount0 delta given sqrt prices and liquidity.
/// Returns a signed integer: positive if adding token0, negative if removing.
pub fn get_amount0_delta(sqrt_ratio_a_x96: U256, sqrt_ratio_b_x96: U256, liquidity: I256) -> I256 {
    let (sqrt_lower, sqrt_upper) = if sqrt_ratio_a_x96 < sqrt_ratio_b_x96 {
        (sqrt_ratio_a_x96, sqrt_ratio_b_x96)
    } else {
        (sqrt_ratio_b_x96, sqrt_ratio_a_x96)
    };

    if liquidity == I256::ZERO || sqrt_lower == sqrt_upper {
        return I256::ZERO;
    }

    // Formula: liquidity * (sqrt_upper - sqrt_lower) / (sqrt_upper * sqrt_lower)
    let liquidity_unsigned = U256::try_from(liquidity.abs()).unwrap_or(U256::ZERO);
    let numerator = liquidity_unsigned * (sqrt_upper - sqrt_lower);
    let denominator = sqrt_upper * sqrt_lower;
    let quotient = numerator / denominator;

    if liquidity.is_negative() {
        -I256::try_from(quotient).unwrap_or(I256::ZERO)
    } else {
        I256::try_from(quotient).unwrap_or(I256::ZERO)
    }
}

/// Computes the amount1 delta given sqrt prices and liquidity.
/// Returns a signed integer: positive if adding token1, negative if removing.
pub fn get_amount1_delta(sqrt_ratio_a_x96: U256, sqrt_ratio_b_x96: U256, liquidity: I256) -> I256 {
    let (sqrt_lower, sqrt_upper) = if sqrt_ratio_a_x96 < sqrt_ratio_b_x96 {
        (sqrt_ratio_a_x96, sqrt_ratio_b_x96)
    } else {
        (sqrt_ratio_b_x96, sqrt_ratio_a_x96)
    };

    if liquidity == I256::ZERO || sqrt_lower == sqrt_upper {
        return I256::ZERO;
    }

    // Formula: liquidity * (sqrt_upper - sqrt_lower)
    let liquidity_unsigned = U256::try_from(liquidity.abs()).unwrap_or(U256::ZERO);
    let diff = sqrt_upper - sqrt_lower;
    let amount = liquidity_unsigned * diff / U256::from(1u128 << 96);

    if liquidity.is_negative() {
        -I256::try_from(amount).unwrap_or(I256::ZERO)
    } else {
        I256::try_from(amount).unwrap_or(I256::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount0_delta() {
        let l = I256::from(1000u128);
        let sqrt_a = U256::from(10u128 << 96);
        let sqrt_b = U256::from(20u128 << 96);
        let result = get_amount0_delta(sqrt_a, sqrt_b, l);
        assert!(result > I256::ZERO);
    }

    #[test]
    fn test_amount1_delta() {
        let l = I256::from(1000u128);
        let sqrt_a = U256::from(10u128 << 96);
        let sqrt_b = U256::from(20u128 << 96);
        let result = get_amount1_delta(sqrt_a, sqrt_b, l);
        assert!(result > I256::ZERO);
    }
}
