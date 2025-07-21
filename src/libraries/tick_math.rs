use alloy_primitives::{I256, U256};

/// Constants for tick math
pub const MIN_TICK: i32 = -887272;
pub const MAX_TICK: i32 = 887272;
pub const MIN_SQRT_RATIO: U256 = U256::from_limbs([4295128739u64, 0, 0, 0]);
pub const MAX_SQRT_RATIO: U256 = U256::from_limbs([
    1461446703485210103287273052203988822378723970342u64,
    0,
    0,
    0,
]);

/// Returns sqrt(price) as a Q64.96 for a given tick
pub fn get_sqrt_ratio_at_tick(tick: i32) -> U256 {
    assert!(tick >= MIN_TICK && tick <= MAX_TICK, "T");

    // This is a simplified version for demonstration.
    // For real simulation, port the full Uniswap V3 logic:
    // https://github.com/Uniswap/v3-core/blob/main/contracts/libraries/TickMath.sol

    // NOTE: The following is NOT the correct formula, it is a placeholder.
    // You MUST port the correct fixed-point math for real use.
    if tick == 0 {
        U256::from(79228162514264337593543950336u128) // 1 << 96
    } else if tick > 0 {
        U256::from(79228162514264337593543950336u128 + (tick as u128) * 1000)
    } else {
        U256::from(79228162514264337593543950336u128 - ((-tick) as u128) * 1000)
    }
}

/// Returns the greatest tick value such that get_sqrt_ratio_at_tick(tick) <= sqrtPriceX96
pub fn get_tick_at_sqrt_ratio(sqrt_price_x96: U256) -> i32 {
    // This function is nontrivial and should be ported from TickMath.sol.
    // Here we give a placeholder for demonstration.
    let base = U256::from(79228162514264337593543950336u128);
    if sqrt_price_x96 == base {
        0
    } else if sqrt_price_x96 > base {
        ((sqrt_price_x96 - base) / U256::from(1000)).as_limbs()[0] as i32
    } else {
        -((base - sqrt_price_x96) / U256::from(1000)).as_limbs()[0] as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_0() {
        let sqrt = get_sqrt_ratio_at_tick(0);
        assert_eq!(sqrt, U256::from(79228162514264337593543950336u128));
        let tick = get_tick_at_sqrt_ratio(sqrt);
        assert_eq!(tick, 0);
    }
}
