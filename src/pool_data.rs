use alloy_primitives::{Address, Bytes, I256, U256};

#[derive(Clone, Debug)]
pub struct Slot0 {
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub observation_index: u16,
    pub observation_cardinality: u16,
    pub observation_cardinality_next: u16,
    pub fee_protocol: u8,
}

#[derive(Clone, Debug)]
pub struct ProtocolFees {
    pub token0: u128,
    pub token1: u128,
}

#[derive(Clone, Debug, Default)]
pub struct TickInfo {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
    pub fee_growth_outside0_x128: U256,
    pub fee_growth_outside1_x128: U256,
}

#[derive(Clone, Debug)]
pub struct SwapParams {
    pub recipient: Address,
    pub zero_for_one: bool,
    pub amount_specified: I256,
    pub sqrt_price_limit_x96: U256,
    pub data: Bytes,
}

#[derive(Clone, Debug)]
pub struct SwapResult {
    pub amount0: I256,
    pub amount1: I256,
}
