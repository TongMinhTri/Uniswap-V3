use alloy_primitives::{I256, U256};

use crate::libraries::{
    error::UniswapV3MathError,
    full_math::{mul_div, mul_div_rounding_up},
    sqrt_price_math::{
        _get_amount_0_delta, _get_amount_1_delta, get_next_sqrt_price_from_input,
        get_next_sqrt_price_from_output,
    },
};

pub fn compute_swap_step(
    sqrt_ratio_current_x_96: U256,
    sqrt_ratio_target_x_96: U256,
    liquidity: u128,
    amount_remaining: I256,
    fee_pips: u32,
) -> Result<(U256, U256, U256, U256), UniswapV3MathError> {
    let zero_for_one = sqrt_ratio_current_x_96 >= sqrt_ratio_target_x_96;
    let exact_in = amount_remaining >= I256::ZERO;

    let sqrt_ratio_next_x_96: U256;
    let mut amount_in = U256::ZERO;
    let mut amount_out = U256::ZERO;

    if exact_in {
        let amount_remaining_less_fee = mul_div(
            amount_remaining.into_raw(),
            U256::from(1e6 as u32 - fee_pips),
            U256::from_limbs([1000000, 0, 0, 0]),
        )?;

        amount_in = if zero_for_one {
            _get_amount_0_delta(
                sqrt_ratio_target_x_96,
                sqrt_ratio_current_x_96,
                liquidity,
                true,
            )?
        } else {
            _get_amount_1_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_target_x_96,
                liquidity,
                true,
            )?
        };

        if amount_remaining_less_fee >= amount_in {
            sqrt_ratio_next_x_96 = sqrt_ratio_target_x_96;
        } else {
            sqrt_ratio_next_x_96 = get_next_sqrt_price_from_input(
                sqrt_ratio_current_x_96,
                liquidity,
                amount_remaining_less_fee,
                zero_for_one,
            )?;
        }
    } else {
        amount_out = if zero_for_one {
            _get_amount_1_delta(
                sqrt_ratio_target_x_96,
                sqrt_ratio_current_x_96,
                liquidity,
                false,
            )?
        } else {
            _get_amount_0_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_target_x_96,
                liquidity,
                false,
            )?
        };

        sqrt_ratio_next_x_96 = if (-amount_remaining).into_raw() >= amount_out {
            sqrt_ratio_target_x_96
        } else {
            get_next_sqrt_price_from_output(
                sqrt_ratio_current_x_96,
                liquidity,
                (-amount_remaining).into_raw(),
                zero_for_one,
            )?
        };
    }

    let max = sqrt_ratio_target_x_96 == sqrt_ratio_next_x_96;

    if zero_for_one {
        if !max || !exact_in {
            amount_in = _get_amount_0_delta(
                sqrt_ratio_next_x_96,
                sqrt_ratio_current_x_96,
                liquidity,
                true,
            )?
        }

        if !max || exact_in {
            amount_out = _get_amount_1_delta(
                sqrt_ratio_next_x_96,
                sqrt_ratio_current_x_96,
                liquidity,
                false,
            )?
        }
    } else {
        if !max || !exact_in {
            amount_in = _get_amount_1_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_next_x_96,
                liquidity,
                true,
            )?
        }

        if !max || exact_in {
            amount_out = _get_amount_0_delta(
                sqrt_ratio_current_x_96,
                sqrt_ratio_next_x_96,
                liquidity,
                false,
            )?
        }
    }

    if !exact_in && amount_out > (-amount_remaining).into_raw() {
        amount_out = (-amount_remaining).into_raw();
    }

    if exact_in && sqrt_ratio_next_x_96 != sqrt_ratio_target_x_96 {
        let fee_amount = amount_remaining.into_raw() - amount_in;
        Ok((sqrt_ratio_next_x_96, amount_in, amount_out, fee_amount))
    } else {
        let fee_amount = mul_div_rounding_up(
            amount_in,
            U256::from(fee_pips),
            U256::from(1e6 as u32 - fee_pips),
        )?;

        Ok((sqrt_ratio_next_x_96, amount_in, amount_out, fee_amount))
    }
}

#[cfg(test)]
mod test {

    use crate::libraries::U256_1;
    use crate::sqrt_price_math::{get_next_sqrt_price_from_input, get_next_sqrt_price_from_output};
    use crate::swap_math::compute_swap_step;
    use alloy_primitives::{I256, U256};
    use std::str::FromStr;

    #[allow(unused)]
    #[test]
    fn test_compute_swap_step() {
        //------------------------------------------------------------

        //exact amount in that gets capped at price target in one for zero
        let price = U256::from_str("79228162514264337593543950336").unwrap();
        let price_target = U256::from_str("79623317895830914510639640423").unwrap();
        let liquidity = 2e18 as u128;
        let amount = I256::from_str("1000000000000000000").unwrap();
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(
            sqrt_p,
            U256::from_str("79623317895830914510639640423").unwrap()
        );

        assert_eq!(amount_in, U256::from_str("9975124224178055").unwrap());
        assert_eq!(fee_amount, U256::from_str("5988667735148").unwrap());
        assert_eq!(amount_out, U256::from_str("9925619580021728").unwrap());

        assert!(amount_in + fee_amount < U256::from_limbs(*amount.as_limbs()));

        let price_after_whole_input_amount =
            get_next_sqrt_price_from_input(price, liquidity, amount_in, zero_for_one).unwrap();

        assert_eq!(sqrt_p, price_target);
        assert!(sqrt_p < price_after_whole_input_amount);

        //------------------------------------------------------------

        //exact amount out that gets capped at price target in one for zero
        let price = U256::from_str("79228162514264337593543950336").unwrap();
        let price_target = U256::from_str("79623317895830914510639640423").unwrap();
        let liquidity = 2e18 as u128;
        let amount = I256::from_str("-1000000000000000000").unwrap();
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(amount_in, U256::from_str("9975124224178055").unwrap());
        assert_eq!(fee_amount, U256::from_str("5988667735148").unwrap());
        assert_eq!(amount_out, U256::from_str("9925619580021728").unwrap());
        assert!(amount_out < (amount * -I256::ONE).into_raw());

        assert!(amount_in + fee_amount < U256::from_limbs(*amount.as_limbs()));

        let price_after_whole_output_amount = get_next_sqrt_price_from_output(
            price,
            liquidity,
            (amount * -I256::ONE).into_raw(),
            zero_for_one,
        )
        .unwrap();

        assert_eq!(sqrt_p, price_target);
        assert!(sqrt_p < price_after_whole_output_amount);

        //------------------------------------------------------------

        //exact amount in that is fully spent in one for zero
        let price = U256::from_str("79228162514264337593543950336").unwrap();
        let price_target = U256::from_str("0xe6666666666666666666666666").unwrap();
        let liquidity = 2e18 as u128;
        let amount = I256::from_str("1000000000000000000").unwrap();
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(amount_in, U256::from_str("999400000000000000").unwrap());
        assert_eq!(fee_amount, U256::from_str("600000000000000").unwrap());
        assert_eq!(amount_out, U256::from_str("666399946655997866").unwrap());
        assert_eq!(amount_in + fee_amount, amount.into_raw());

        let price_after_whole_input_amount_less_fee = get_next_sqrt_price_from_input(
            price,
            liquidity,
            (amount - I256::from_raw(fee_amount)).into_raw(),
            zero_for_one,
        )
        .unwrap();

        assert!(sqrt_p < price_target);
        assert_eq!(sqrt_p, price_after_whole_input_amount_less_fee);

        //------------------------------------------------------------

        //exact amount out that is fully received in one for zero
        let price = U256::from_str("79228162514264337593543950336").unwrap();
        let price_target = U256::from_str("792281625142643375935439503360").unwrap();
        let liquidity = 2e18 as u128;
        let amount = I256::from_str("1000000000000000000").unwrap() * -I256::ONE;
        let fee = 600;
        let zero_for_one = false;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount, fee).unwrap();

        assert_eq!(amount_in, U256::from_str("2000000000000000000").unwrap());
        assert_eq!(fee_amount, U256::from_str("1200720432259356").unwrap());
        assert_eq!(amount_out, (amount * -I256::ONE).into_raw());

        let price_after_whole_output_amount = get_next_sqrt_price_from_output(
            price,
            liquidity,
            (amount * -I256::ONE).into_raw(),
            zero_for_one,
        )
        .unwrap();
        //sqrtPrice 158456325028528675187087900672
        //price_after_whole_output_amount Should be: 158456325028528675187087900672
        // sqrtp: 158456325028528675187087900672, price_after_whole output amount: 118842243771396506390315925504

        assert!(sqrt_p < price_target);
        //TODO:FIXME: failing
        println!(
            "sqrtp: {:?}, price_after_whole output amount: {:?}",
            sqrt_p, price_after_whole_output_amount
        );
        assert_eq!(sqrt_p, price_after_whole_output_amount);

        //------------------------------------------------------------

        //amount out is capped at the desired amount out
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_str("417332158212080721273783715441582").unwrap(),
            U256::from_str("1452870262520218020823638996").unwrap(),
            159344665391607089467575320103_u128,
            I256::from_str("-1").unwrap(),
            1,
        )
        .unwrap();

        assert_eq!(amount_in, U256::from_str("1").unwrap());
        assert_eq!(fee_amount, U256::from_str("1").unwrap());
        assert_eq!(amount_out, U256::from_str("1").unwrap());
        assert_eq!(
            sqrt_p,
            U256::from_str("417332158212080721273783715441581").unwrap()
        );

        //------------------------------------------------------------

        //target price of 1 uses partial input amount
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_str("2").unwrap(),
            U256::from_str("1").unwrap(),
            1_u128,
            I256::from_str("3915081100057732413702495386755767").unwrap(),
            1,
        )
        .unwrap();

        assert_eq!(
            amount_in,
            U256::from_str("39614081257132168796771975168").unwrap()
        );
        assert_eq!(
            fee_amount,
            U256::from_str("39614120871253040049813").unwrap()
        );
        assert!(
            amount_in + fee_amount < U256::from_str("3915081100057732413702495386755767").unwrap()
        );
        assert_eq!(amount_out, U256::from_str("0").unwrap());

        assert_eq!(sqrt_p, U256::from_str("1").unwrap());

        //------------------------------------------------------------

        //entire input amount taken as fee
        let (sqrt_p, amount_in, amount_out, fee_amount) = compute_swap_step(
            U256::from_str("2413").unwrap(),
            U256::from_str("79887613182836312").unwrap(),
            1985041575832132834610021537970_u128,
            I256::from_str("10").unwrap(),
            1872,
        )
        .unwrap();

        assert_eq!(amount_in, U256::from_str("0").unwrap());
        assert_eq!(fee_amount, U256::from_str("10").unwrap());
        assert_eq!(amount_out, U256::from_str("0").unwrap());
        assert_eq!(sqrt_p, U256::from_str("2413").unwrap());

        //------------------------------------------------------------

        //handles intermediate insufficient liquidity in zero for one exact output case

        let price = U256::from_str("20282409603651670423947251286016").unwrap();
        let price_target = price * U256::from(11) / U256::from(10);
        let liquidity = 1024;
        // virtual reserves of one are only 4
        let amount_remaining = -I256::from_limbs(*U256::from(4).as_limbs());
        let fee = 3000;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount_remaining, fee).unwrap();

        assert_eq!(amount_out, U256::ZERO);
        assert_eq!(sqrt_p, price_target);
        assert_eq!(amount_in, U256::from(26215));
        assert_eq!(fee_amount, U256::from(79));

        //------------------------------------------------------------

        //handles intermediate insufficient liquidity in one for zero exact output case

        let price = U256::from_str("20282409603651670423947251286016").unwrap();

        let price_target = price * U256::from(9) / U256::from(10);
        let liquidity = 1024;
        // virtual reserves of zero are only 262144
        let amount_remaining = -I256::from_limbs(*U256::from(263000).as_limbs());
        let fee = 3000;

        let (sqrt_p, amount_in, amount_out, fee_amount) =
            compute_swap_step(price, price_target, liquidity, amount_remaining, fee).unwrap();

        assert_eq!(amount_out, U256::from(26214));
        assert_eq!(sqrt_p, price_target);
        assert_eq!(amount_in, U256_1);
        assert_eq!(fee_amount, U256_1);
    }
}
