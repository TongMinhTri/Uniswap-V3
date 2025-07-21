use alloy_primitives::{I256, U256};

pub fn add_delta(liquidity: U256, delta: I256) -> U256 {
    if delta.is_negative() {
        let delta_abs = U256::try_from(delta.abs()).unwrap_or(U256::ZERO);
        assert!(liquidity >= delta_abs, "liquidity underflow");
        liquidity - delta_abs
    } else {
        liquidity + U256::try_from(delta).unwrap_or(U256::ZERO)
    }
}
