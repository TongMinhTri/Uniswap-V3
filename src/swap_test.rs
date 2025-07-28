use super::*;
use alloy_primitives::Bytes;

#[test]
fn test_swap_1() {
    let mut pool = UniswapV3Pool::from_json_file(
        "snapshots/54994241/Pan.V3.USD1.MERL.0xb604D4E46509FE1c1ef70Ab4a4941d12a49Dbd76.json",
    );

    let params = SwapParams {
        recipient: Address::from_str("0x13f4ea83d0bd40e75c8222255bc855a974568dd4").unwrap(),
        zero_for_one: false,
        amount_specified: I256::from_str("25349109482797066497").unwrap(),
        sqrt_price_limit_x96: U256::from_str("1461446703485210103287273052203988822378723970341")
            .unwrap(),
        data: Bytes::from(vec![0u8; 32]),
    };

    let result = pool.swap(params).unwrap();

    assert_eq!(
        (result.amount0, result.amount1),
        (
            I256::from_dec_str("-3229851649125690539").unwrap(),
            I256::from_dec_str("25349109482797066497").unwrap()
        ),
        "amounts are incorrect"
    );
}

#[test]
fn test_swap_2() {
    let mut pool = UniswapV3Pool::from_json_file(
        "snapshots/55002250/Pan.V3.USD1.MERL.0xb604D4E46509FE1c1ef70Ab4a4941d12a49Dbd76.json",
    );

    let params = SwapParams {
        recipient: Address::from_str("0xf258fcd1a2c216cd3f3303bea930cca1b6350d5d").unwrap(),
        zero_for_one: true,
        amount_specified: I256::from_str("111943783213448371527").unwrap(),
        sqrt_price_limit_x96: U256::from_str("4295128740").unwrap(),
        data: Bytes::from(vec![0u8; 32]),
    };

    let result = pool.swap(params).unwrap();

    assert_eq!(
        (result.amount0, result.amount1),
        (
            I256::from_dec_str("111943783213448371527").unwrap(),
            I256::from_dec_str("-859460639382007394988").unwrap()
        ),
        "amounts are incorrect"
    );
}
