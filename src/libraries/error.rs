use alloy_primitives::ruint::ParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UniswapV3MathError {
    #[error("Invalid sqrt price limit")]
    InvalidSqrtPriceLimit,
    #[error("Specified amount is 0")]
    ZeroAmountSpecified,
    #[error("Denominator is 0")]
    DenominatorIsZero,
    #[error("Result is U256::MAX")]
    ResultIsU256MAX,
    #[error("Sqrt price is 0")]
    SqrtPriceIsZero,
    #[error("Sqrt price is less than or equal to quotient")]
    SqrtPriceIsLteQuotient,
    #[error("Can not get most significant bit or least significant bit on zero value")]
    LiquidityIsZero,
    #[error("require((product = amount * sqrtPX96) / amount == sqrtPX96 && numerator1 > product);")]
    ProductDivAmount,
    #[error("Denominator is less than or equal to prod_1")]
    DenominatorIsLteProdOne,
    #[error("The given tick must be less than, or equal to, the maximum tick")]
    T,
    #[error("Overflow when casting to U160")]
    SafeCastToU160Overflow,
    #[error("Parse error")]
    ParseError(#[from] ParseError),
}
