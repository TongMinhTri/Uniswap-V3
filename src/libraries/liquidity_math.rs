use alloy_primitives::{I256, U256};

/// Adds delta to liquidity, as in Uniswap V3 LiquidityMath.
pub fn add_delta(liquidity: U256, delta: I256) -> U256 {
    if delta.is_negative() {
        let delta_abs = U256::try_from(delta.abs()).unwrap_or(U256::ZERO);
        assert!(liquidity >= delta_abs, "liquidity underflow");
        liquidity - delta_abs
    } else {
        liquidity + U256::try_from(delta).unwrap_or(U256::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_delta_positive() {
        let l = U256::from(1000u128);
        let delta = I256::from(500u128);
        let result = add_delta(l, delta);
        assert_eq!(result, U256::from(1500u128));
    }

    #[test]
    fn test_add_delta_negative() {
        let l = U256::from(1000u128);
        let delta = I256::from(-500i128);
        let result = add_delta(l, delta);
        assert_eq!(result, U256::from(500u128));
    }
}
