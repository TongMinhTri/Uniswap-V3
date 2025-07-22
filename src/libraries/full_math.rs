use std::ops::{Add, BitOrAssign, Div, Mul, MulAssign};

use alloy_primitives::U256;

use crate::libraries::{U256_1, U256_2, U256_3, error::UniswapV3MathError};

pub fn mul_div(a: U256, b: U256, mut denominator: U256) -> Result<U256, UniswapV3MathError> {
    let mm = a.mul_mod(b, U256::MAX);

    let mut prod_0 = a.overflowing_mul(b).0;
    let mut prod_1 = mm
        .overflowing_sub(prod_0)
        .0
        .overflowing_sub(U256::from((mm < prod_0) as u8))
        .0;

    if prod_1 == U256::ZERO {
        if denominator == U256::ZERO {
            return Err(UniswapV3MathError::DenominatorIsZero);
        }
        return Ok(U256::from_limbs(*prod_0.div(denominator).as_limbs()));
    }

    if denominator <= prod_1 {
        return Err(UniswapV3MathError::DenominatorIsLteProdOne);
    }

    let remainder = a.mul_mod(b, denominator);

    prod_1 = prod_1
        .overflowing_sub(U256::from((remainder > prod_0) as u8))
        .0;
    prod_0 = prod_0.overflowing_sub(remainder).0;

    let mut twos = U256::ZERO
        .overflowing_sub(denominator)
        .0
        .bitand(denominator);

    denominator = denominator.wrapping_div(twos);

    prod_0 = prod_0.wrapping_div(twos);

    twos = (U256::ZERO.overflowing_sub(twos).0.wrapping_div(twos)).add(U256_1);

    prod_0.bitor_assign(prod_1 * twos);

    let mut inv = U256_3.mul(denominator).bitxor(U256_2);

    inv.mul_assign(U256_2 - denominator * inv);
    inv.mul_assign(U256_2 - denominator * inv);
    inv.mul_assign(U256_2 - denominator * inv);
    inv.mul_assign(U256_2 - denominator * inv);
    inv.mul_assign(U256_2 - denominator * inv);
    inv.mul_assign(U256_2 - denominator * inv);

    Ok(U256::from_le_slice((prod_0 * inv).as_le_slice()))
}

pub fn mul_div_rounding_up(
    a: U256,
    b: U256,
    denominator: U256,
) -> Result<U256, UniswapV3MathError> {
    let result = mul_div(a, b, denominator)?;

    if a.mul_mod(b, denominator) > U256::ZERO {
        if result == U256::MAX {
            Err(UniswapV3MathError::ResultIsU256MAX)
        } else {
            Ok(result + U256_1)
        }
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use crate::libraries::U256_1;
    use alloy_primitives::U256;
    use std::ops::{Div, Mul, Sub};

    use super::mul_div;

    const Q128: U256 = U256::from_limbs([0, 0, 1, 0]);

    #[test]
    fn test_mul_div() {
        //Revert if the denominator is zero
        let result = mul_div(Q128, U256::from(5), U256::ZERO);
        assert_eq!(result.err().unwrap().to_string(), "Denominator is 0");

        // Revert if the denominator is zero and numerator overflows
        let result = mul_div(Q128, Q128, U256::ZERO);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );

        // Revert if the output overflows uint256
        let result = mul_div(Q128, Q128, U256_1);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );

        // Reverts on overflow with all max inputs
        let result = mul_div(U256::MAX, U256::MAX, U256::MAX.sub(U256_1));
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );

        // All max inputs
        let result = mul_div(U256::MAX, U256::MAX, U256::MAX);
        assert_eq!(result.unwrap(), U256::MAX);

        // Accurate without phantom overflow
        let result = mul_div(
            Q128,
            U256::from(50).mul(Q128).div(U256::from(100)),
            U256::from(150).mul(Q128).div(U256::from(100)),
        );
        assert_eq!(result.unwrap(), Q128.div(U256::from(3)));

        // Accurate with phantom overflow
        let result = mul_div(Q128, U256::from(35).mul(Q128), U256::from(8).mul(Q128));
        assert_eq!(
            result.unwrap(),
            U256::from(4375).mul(Q128).div(U256::from(1000))
        );

        // Accurate with phantom overflow and repeating decimal
        let result = mul_div(Q128, U256::from(1000).mul(Q128), U256::from(3000).mul(Q128));
        assert_eq!(result.unwrap(), Q128.div(U256::from(3)));
    }
}

#[cfg(test)]
mod tests {
    use crate::libraries::U256_1;
    use alloy_primitives::U256;
    use std::ops::{Add, Div, Mul, Sub};

    use super::mul_div_rounding_up;

    const Q128: U256 = U256::from_limbs([0, 0, 1, 0]);

    #[test]
    fn test_mul_div_rounding_up() {
        //Revert if the denominator is zero
        let result = mul_div_rounding_up(Q128, U256::from(5), U256::ZERO);
        assert_eq!(result.err().unwrap().to_string(), "Denominator is 0");

        // Revert if the denominator is zero and numerator overflows
        let result = mul_div_rounding_up(Q128, Q128, U256::ZERO);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );

        // Revert if the output overflows uint256
        let result = mul_div_rounding_up(Q128, Q128, U256_1);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );

        // Reverts on overflow with all max inputs
        let result = mul_div_rounding_up(U256::MAX, U256::MAX, U256::MAX.sub(U256_1));
        assert_eq!(
            result.err().unwrap().to_string(),
            "Denominator is less than or equal to prod_1"
        );

        // Reverts if mulDiv overflows 256 bits after rounding up
        let result = mul_div_rounding_up(
            U256::from_str_radix("535006138814359", 10).unwrap(),
            U256::from_str_radix(
                "432862656469423142931042426214547535783388063929571229938474969",
                10,
            )
            .unwrap(),
            U256::from_str_radix("2", 10).unwrap(),
        );
        assert_eq!(result.err().unwrap().to_string(), "Result is U256::MAX");

        // All max inputs
        let result = mul_div_rounding_up(U256::MAX, U256::MAX, U256::MAX);
        assert_eq!(result.unwrap(), U256::MAX);

        // Accurate without phantom overflow
        let result = mul_div_rounding_up(
            Q128,
            U256::from(50).mul(Q128).div(U256::from(100)),
            U256::from(150).mul(Q128).div(U256::from(100)),
        );
        assert_eq!(result.unwrap(), Q128.div(U256::from(3)).add(U256_1));

        // Accurate with phantom overflow
        let result = mul_div_rounding_up(Q128, U256::from(35).mul(Q128), U256::from(8).mul(Q128));
        assert_eq!(
            result.unwrap(),
            U256::from(4375).mul(Q128).div(U256::from(1000))
        );

        // Accurate with phantom overflow and repeating decimal
        let result =
            mul_div_rounding_up(Q128, U256::from(1000).mul(Q128), U256::from(3000).mul(Q128));
        assert_eq!(result.unwrap(), Q128.div(U256::from(3)).add(U256_1));
    }
}
