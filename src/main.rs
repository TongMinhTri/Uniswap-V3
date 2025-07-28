use alloy_primitives::aliases::{I24, U24};
use alloy_primitives::{Address, I16, I256, U256};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

mod libraries;
mod pool_data;

use libraries::*;
use pool_data::*;

use crate::libraries::error::UniswapV3MathError;

#[derive(Clone, Debug)]
pub struct UniswapV3Pool {
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
    pub tick_spacing: I24,
    pub slot0: Slot0,
    pub fee_growth_global0_x128: U256,
    pub fee_growth_global1_x128: U256,
    pub protocol_fees: ProtocolFees,
    pub liquidity: u128,
    pub ticks: HashMap<i32, TickInfo>,
    pub tick_bitmap: HashMap<I16, U256>,
}

impl UniswapV3Pool {
    pub fn from_json_file(path: &str) -> Self {
        let json_str = fs::read_to_string(path).expect("Cannot read JSON file");
        let json: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");

        let pool = &json["pool"]["store"];
        let token0 = Address::from_str(json["pool"]["token0"].as_str().unwrap()).unwrap();
        let token1 = Address::from_str(json["pool"]["token1"].as_str().unwrap()).unwrap();

        let fee = U24::from_str_radix(pool["fee"].as_str().unwrap().trim_start_matches("0x"), 16)
            .unwrap();
        let tick_spacing = pool["tick_spacing"]
            .as_str()
            .unwrap()
            .parse::<I24>()
            .unwrap();

        let slot0_obj = &pool["slot0"];
        let slot0 = Slot0 {
            sqrt_price_x96: U256::from_str_radix(
                slot0_obj["sqrt_price_x96"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("0x"),
                16,
            )
            .unwrap(),
            tick: slot0_obj["tick"].as_str().unwrap().parse::<i32>().unwrap(),
            observation_index: 0,
            observation_cardinality: 0,
            observation_cardinality_next: 0,
            fee_protocol: u8::from_str_radix(
                slot0_obj["fee_protocol"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("0x"),
                16,
            )
            .unwrap(),
        };

        let fee_growth_global0_x128 = U256::from_str_radix(
            pool["fee_growth_global_0x128"]
                .as_str()
                .unwrap()
                .trim_start_matches("0x"),
            16,
        )
        .unwrap();
        let fee_growth_global1_x128 = U256::from_str_radix(
            pool["fee_growth_global_1x128"]
                .as_str()
                .unwrap()
                .trim_start_matches("0x"),
            16,
        )
        .unwrap();
        let liquidity = u128::from_str_radix(
            pool["liquidity"].as_str().unwrap().trim_start_matches("0x"),
            16,
        )
        .unwrap();

        let mut tick_bitmap = HashMap::new();
        if let Some(map) = pool["tick_bitmap"].as_object() {
            for (k, v) in map.iter() {
                let key = k.parse::<I16>().unwrap();
                let value =
                    U256::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
                tick_bitmap.insert(key, value);
            }
        }

        let mut ticks = HashMap::new();
        if let Some(map) = pool["ticks"].as_object() {
            for (k, v) in map.iter() {
                let key = k.parse::<i32>().unwrap();
                let liquidity_net = i128::from_str(v["liquidity_net"].as_str().unwrap()).unwrap();
                let liquidity_gross = u128::from_str_radix(
                    v["liquidity_gross"]
                        .as_str()
                        .unwrap()
                        .trim_start_matches("0x"),
                    16,
                )
                .unwrap();
                let fee_growth_outside0_x128 = U256::from_str_radix(
                    v["fee_growth_outside_0x128"]
                        .as_str()
                        .unwrap()
                        .trim_start_matches("0x"),
                    16,
                )
                .unwrap();
                let fee_growth_outside1_x128 = U256::from_str_radix(
                    v["fee_growth_outside_1x128"]
                        .as_str()
                        .unwrap()
                        .trim_start_matches("0x"),
                    16,
                )
                .unwrap();
                ticks.insert(
                    key,
                    TickInfo {
                        liquidity_gross,
                        liquidity_net,
                        fee_growth_outside0_x128,
                        fee_growth_outside1_x128,
                    },
                );
            }
        }

        let protocol_fees = ProtocolFees {
            token0: u128::from_str_radix(
                pool["protocol_fees"]["token0"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("0x"),
                16,
            )
            .unwrap(),
            token1: u128::from_str_radix(
                pool["protocol_fees"]["token1"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("0x"),
                16,
            )
            .unwrap(),
        };

        Self {
            token0,
            token1,
            fee,
            tick_spacing,
            slot0,
            fee_growth_global0_x128,
            fee_growth_global1_x128,
            protocol_fees,
            liquidity,
            ticks,
            tick_bitmap,
        }
    }
    pub fn swap(&mut self, params: SwapParams) -> Result<SwapResult, UniswapV3MathError> {
        if params.amount_specified == I256::ZERO {
            return Err(UniswapV3MathError::ZeroAmountSpecified);
        }

        let mut amount_specified_remaining = params.amount_specified;
        let mut amount_calculated = I256::ZERO;
        let mut sqrt_price_x96 = self.slot0.sqrt_price_x96;
        let mut tick = self.slot0.tick;
        let mut liquidity = self.liquidity;

        let exact_input = params.amount_specified > I256::ZERO;

        // Collect all initialized tick indices and sort for efficient searching
        let mut all_ticks: Vec<i32> = self.ticks.keys().copied().collect();
        all_ticks.sort();

        while amount_specified_remaining != I256::ZERO
            && ((params.zero_for_one && sqrt_price_x96 > params.sqrt_price_limit_x96)
                || (!params.zero_for_one && sqrt_price_x96 < params.sqrt_price_limit_x96))
        {
            // 1. Find the next initialized tick in the direction
            let next_tick_opt = if params.zero_for_one {
                all_ticks.iter().filter(|&&t| t < tick).max().copied()
            } else {
                all_ticks.iter().filter(|&&t| t > tick).min().copied()
            };

            // If no more initialized ticks, use boundary tick
            let (next_tick, sqrt_price_next_x96) = if let Some(next_tick) = next_tick_opt {
                (
                    next_tick,
                    tick_math::get_sqrt_ratio_at_tick(next_tick).unwrap(),
                )
            } else if params.zero_for_one {
                (
                    tick_math::MIN_TICK,
                    tick_math::get_sqrt_ratio_at_tick(tick_math::MIN_TICK).unwrap(),
                )
            } else {
                (
                    tick_math::MAX_TICK,
                    tick_math::get_sqrt_ratio_at_tick(tick_math::MAX_TICK).unwrap(),
                )
            };

            // 2. Set the target price for this step
            let target_price_x96 = if params.zero_for_one {
                if sqrt_price_next_x96 < params.sqrt_price_limit_x96 {
                    params.sqrt_price_limit_x96
                } else {
                    sqrt_price_next_x96
                }
            } else {
                if sqrt_price_next_x96 > params.sqrt_price_limit_x96 {
                    params.sqrt_price_limit_x96
                } else {
                    sqrt_price_next_x96
                }
            };

            // 3. Compute the swap step
            let (new_sqrt_price_x96, amount_in, amount_out, fee_amount) =
                swap_math::compute_swap_step(
                    sqrt_price_x96,
                    target_price_x96,
                    liquidity,
                    amount_specified_remaining,
                    self.fee.as_limbs()[0] as u32,
                )
                .unwrap();

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

            // 4. If we reached the next tick, update liquidity
            if sqrt_price_x96 == sqrt_price_next_x96 {
                tick = if params.zero_for_one {
                    next_tick - 1
                } else {
                    next_tick
                };
                if let Some(tick_info) = self.ticks.get(&next_tick) {
                    // Add the signed liquidity_net directly!
                    if params.zero_for_one {
                        liquidity = (liquidity as i128 + tick_info.liquidity_net) as u128;
                    } else {
                        liquidity = (liquidity as i128 + tick_info.liquidity_net) as u128;
                    }
                }
            } else {
                // Not crossing a tick, update tick according to current sqrt_price_x96
                tick = tick_math::get_tick_at_sqrt_ratio(sqrt_price_x96).unwrap();
            }
        }

        // Final amount0, amount1
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

        // Update pool state
        self.slot0.sqrt_price_x96 = sqrt_price_x96;
        self.slot0.tick = tick;
        self.liquidity = liquidity;

        Ok(SwapResult { amount0, amount1 })
    }
}

fn main() {}

#[cfg(test)]
mod swap_test;
