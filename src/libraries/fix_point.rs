use alloy_primitives::U256;

/// Q128.128 fixed point constant
pub const Q128: U256 = U256::from_limbs([0, 1, 0, 0]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q128() {
        assert_eq!(Q128, U256::from(1u128 << 128));
    }
}
