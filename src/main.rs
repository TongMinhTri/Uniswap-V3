use alloy_primitives::{Address, Bytes, I16, I32, I256, U8, U16, U32, U256};
use std::collections::HashMap;

mod libraries;

use libraries::*;

#[derive(Clone, Debug)]
pub struct Slot0 {
    pub sqrt_price_x96: U256,
    pub tick: I32,
    pub observation_index: U16,
    pub observation_cardinality: U16,
    pub observation_cardinality_next: U16,
    pub fee_protocol: U8,
    pub unlocked: bool,
}

#[derive(Clone, Debug)]
pub struct ProtocolFees {
    pub token0: U256,
    pub token1: U256,
}

#[derive(Clone, Debug, Default)]
pub struct TickInfo {
    pub liquidity_gross: U256,
    pub liquidity_net: I256,
    pub fee_growth_outside0_x128: U256,
    pub fee_growth_outside1_x128: U256,
    pub tick_cumulative_outside: I256,
    pub seconds_per_liquidity_outside_x128: U256,
    pub seconds_outside: U32,
    pub initialized: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PositionInfo {
    pub liquidity: I256,
    pub tokens_owed0: U256,
    pub tokens_owed1: U256,
    pub fee_growth_inside0_last_x128: U256,
    pub fee_growth_inside1_last_x128: U256,
}

pub struct SwapParams {
    pub recipient: Address,
    pub zero_for_one: bool,
    pub amount_specified: I256,
    pub sqrt_price_limit_x96: U256,
    pub data: Bytes,
}

pub struct SwapResult {
    pub amount0: I256,
    pub amount1: I256,
}

pub struct UniswapV3Pool {
    pub token0: Address,
    pub token1: Address,
    pub fee: U32,
    pub tick_spacing: I32,
    pub slot0: Slot0,
    pub fee_growth_global0_x128: U256,
    pub fee_growth_global1_x128: U256,
    pub protocol_fees: ProtocolFees,
    pub liquidity: U256,
    pub ticks: HashMap<I32, TickInfo>,
    pub tick_bimap: HashMap<I16, U256>,
}

impl UniswapV3Pool {
    pub fn new(
        token0: Address,
        token1: Address,
        fee: U32,
        tick_spacing: I32,
        slot0: Slot0,
    ) -> Self {
        Self {
            token0,
            token1,
            fee,
            tick_spacing,
            slot0,
            fee_growth_global0_x128: U256::ZERO,
            fee_growth_global1_x128: U256::ZERO,
            protocol_fees: ProtocolFees {
                token0: U256::ZERO,
                token1: U256::ZERO,
            },
            liquidity: U256::ZERO,
            ticks: HashMap::new(),
            tick_bimap: HashMap::new(),
        }
    }

    pub fn swap(&mut self, params: SwapParams) -> SwapResult {
        assert!(params.amount_specified != I256::ZERO, "AS");
        assert!(self.slot0.unlocked, "LOK");

        let slot0 = self.slot0.clone();
        let exact_input = params.amount_specified > I256::ZERO;

        let valid_limit = if params.zero_for_one {
            params.sqrt_price_limit_x96 < slot0.sqrt_price_x96
                && params.sqrt_price_limit_x96 > tick_math::MIN_SQRT_RATIO
        } else {
            params.sqrt_price_limit_x96 > slot0.sqrt_price_x96
                && params.sqrt_price_limit_x96 < tick_math::MAX_SQRT_RATIO
        };
        assert!(valid_limit, "SPL");

        self.slot0.unlocked = false;

        let mut amount_specified_remaining = params.amount_specified;
        let mut amount_calculated = I256::ZERO;
        let mut sqrt_price_x96 = slot0.sqrt_price_x96;
        let tick = slot0.tick;
        let liquidity = self.liquidity;

        while amount_specified_remaining != I256::ZERO
            && sqrt_price_x96 != params.sqrt_price_limit_x96
        {
            // For demo: just use the edge tick, in a real implementation you'd scan for the next initialized tick
            let next_tick = if params.zero_for_one {
                tick_math::MIN_TICK
            } else {
                tick_math::MAX_TICK
            };
            let sqrt_price_next_x96 = tick_math::get_sqrt_ratio_at_tick(next_tick);

            let target_price_x96 = if (params.zero_for_one
                && sqrt_price_next_x96 < params.sqrt_price_limit_x96)
                || (!params.zero_for_one && sqrt_price_next_x96 > params.sqrt_price_limit_x96)
            {
                params.sqrt_price_limit_x96
            } else {
                sqrt_price_next_x96
            };

            let (new_sqrt_price_x96, amount_in, amount_out, fee_amount) =
                swap_math::compute_swap_step(
                    sqrt_price_x96,
                    target_price_x96,
                    liquidity,
                    amount_specified_remaining,
                    self.fee.as_limbs()[0] as u32,
                );

            let step_amount_in = I256::try_from(amount_in).unwrap_or(I256::ZERO);
            let step_amount_out = I256::try_from(amount_out).unwrap_or(I256::ZERO);
            let step_fee = I256::try_from(fee_amount).unwrap_or(I256::ZERO);

            if exact_input {
                amount_specified_remaining -= step_amount_in + step_fee;
                amount_calculated -= step_amount_out;
            } else {
                amount_specified_remaining += step_amount_out;
                amount_calculated += step_amount_in + step_fee;
            }

            sqrt_price_x96 = new_sqrt_price_x96;

            break;
        }

        let (amount0, amount1) = if params.zero_for_one == exact_input {
            (
                params.amount_specified - amount_specified_remaining,
                amount_calculated,
            )
        } else {
            (
                amount_calculated,
                params.amount_specified - amount_specified_remaining,
            )
        };

        self.slot0.sqrt_price_x96 = sqrt_price_x96;
        self.slot0.tick = tick;
        self.slot0.unlocked = true;

        SwapResult { amount0, amount1 }
    }
}

fn main() {
    let mut pool = UniswapV3Pool::from_json_file(
        "./src/v3.0xb604D4E46509FE1c1ef70Ab4a4941d12a49Dbd76.USD1_MERL.json",
    );

    let params = SwapParams {
        recipient: Address::ZERO,
        zero_for_one: true,
        amount_specified: I256::from(1035756642273342086484),
        sqrt_price_limit_x96: libraries::tick_math::MIN_SQRT_RATIO + U256::from(1u64),
        data: Bytes::default(),
    };

    let result = pool.swap(params);

    println!(
        "Swap result: amount0 = {}, amount1 = {}",
        result.amount0, result.amount1
    );
}
