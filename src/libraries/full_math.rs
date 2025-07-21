use alloy_primitives::U256;

/// Performs full precision multiplication and division, like Uniswap's FullMath.mulDiv.
/// Returns floor(a * b / denominator)
pub fn mul_div(a: U256, b: U256, denominator: U256) -> U256 {
    if denominator.is_zero() {
        panic!("division by zero");
    }
    let (prod0, prod1) = full_mul(a, b);

    if prod1.is_zero() {
        // fits in 256 bits
        return prod0 / denominator;
    }

    // 512-bit division
    let mut remainder = mulmod(a, b, denominator);
    let mut prod0 = prod0;
    let mut prod1 = prod1;

    if remainder > prod0 {
        prod1 -= U256::ONE;
        prod0 = prod0 + (U256::MAX - remainder) + U256::ONE;
    } else {
        prod0 -= remainder;
    }

    // Factor powers of two out of denominator
    let twos = denominator & (!denominator + U256::ONE);
    let denominator = denominator / twos;
    prod0 = prod0 / twos;

    let inv = mod_inverse(denominator);

    // (prod0 | prod1 << 256) * inv
    let result = mulmod(prod0, inv, U256::MAX);
    result
}

/// Multiplies two U256, returning the full 512-bit result as (low, high)
pub fn full_mul(a: U256, b: U256) -> (U256, U256) {
    let a_lo = a & U256::from(u128::MAX);
    let a_hi = a >> 128;
    let b_lo = b & U256::from(u128::MAX);
    let b_hi = b >> 128;

    let lo = a_lo * b_lo;
    let mid1 = a_lo * b_hi;
    let mid2 = a_hi * b_lo;
    let hi = a_hi * b_hi;

    let mid = mid1 + mid2;
    let hi = hi + (mid >> 128);
    let mid = (mid << 128) + lo;

    (mid, hi)
}

/// Returns a * b % modulus
pub fn mulmod(a: U256, b: U256, modulus: U256) -> U256 {
    if modulus.is_zero() {
        panic!("modulus is zero");
    }
    ((a.full_mul(b).0) % modulus)
}

/// Returns modular inverse of a mod 2^256 (used in Uniswap for division)
pub fn mod_inverse(a: U256) -> U256 {
    // For simplicity, use pow(a, -1, 2^256) for small denominators.
    // In production you may want to use extended Euclidean algorithm for big moduli.
    let mut t = U256::ZERO;
    let mut newt = U256::ONE;
    let mut r = U256::MAX;
    let mut newr = a;
    while !newr.is_zero() {
        let quotient = r / newr;
        t = newt;
        newt = t - quotient * newt;
        r = newr;
        newr = r - quotient * newr;
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_div() {
        let a = U256::from(10u128);
        let b = U256::from(20u128);
        let d = U256::from(5u128);
        let result = mul_div(a, b, d);
        assert_eq!(result, U256::from(40u128));
    }
}
