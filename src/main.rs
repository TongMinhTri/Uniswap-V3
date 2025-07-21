use alloy_primitives::{Address, I256, U256};
use std::collections::HashMap;

mod libraries;

use libraries::*;

/// Pool slot0 state
#[derive(Clone, Debug)]
pub struct Slot0 {
    pub sqrt_price_x96: U256,
    pub tick: i32,
    pub observation_index: u16,
    pub observation_cardinality: u16,
    pub observation_cardinality_next: u16,
    pub fee_protocol: u8,
    pub unlocked: bool,
}

/// Protocol fees
#[derive(Clone, Debug)]
pub struct ProtocolFees {
    pub token0: U256,
    pub token1: U256,
}

/// Tick information
#[derive(Clone, Debug, Default)]
pub struct TickInfo {
    pub liquidity_gross: U256,
    pub liquidity_net: I256,
    pub fee_growth_outside0_x128: U256,
    pub fee_growth_outside1_x128: U256,
    pub tick_cumulative_outside: I256,
    pub seconds_per_liquidity_outside_x128: U256,
    pub seconds_outside: u32,
    pub initialized: bool,
}

/// Position information
#[derive(Clone, Debug, Default)]
pub struct PositionInfo {
    pub liquidity: I256,
    pub tokens_owed0: U256,
    pub tokens_owed1: U256,
    pub fee_growth_inside0_last_x128: U256,
    pub fee_growth_inside1_last_x128: U256,
}

/// Swap parameters
pub struct SwapParams {
    pub recipient: Address,
    pub zero_for_one: bool,
    pub amount_specified: I256,
    pub sqrt_price_limit_x96: U256,
}

/// Swap result
pub struct SwapResult {
    pub amount0: I256,
    pub amount1: I256,
}

pub struct UniswapV3Pool {
    pub factory: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
    pub max_liquidity_per_tick: U256,
    pub slot0: Slot0,
    pub fee_growth_global0_x128: U256,
    pub fee_growth_global1_x128: U256,
    pub protocol_fees: ProtocolFees,
    pub liquidity: U256,
    pub ticks: HashMap<i32, TickInfo>,
    pub positions: HashMap<(Address, i32, i32), PositionInfo>,
}

impl UniswapV3Pool {
    pub fn new(
        factory: Address,
        token0: Address,
        token1: Address,
        fee: u32,
        tick_spacing: i32,
        max_liquidity_per_tick: U256,
        slot0: Slot0,
    ) -> Self {
        Self {
            factory,
            token0,
            token1,
            fee,
            tick_spacing,
            max_liquidity_per_tick,
            slot0,
            fee_growth_global0_x128: U256::ZERO,
            fee_growth_global1_x128: U256::ZERO,
            protocol_fees: ProtocolFees {
                token0: U256::ZERO,
                token1: U256::ZERO,
            },
            liquidity: U256::ZERO,
            ticks: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    /// Add liquidity to a position
    pub fn mint(&mut self, owner: Address, tick_lower: i32, tick_upper: i32, amount: U256) {
        let position = self
            .positions
            .entry((owner, tick_lower, tick_upper))
            .or_insert(PositionInfo::default());
        position.liquidity += I256::try_from(amount).unwrap_or(I256::ZERO);
        self.liquidity += amount;
        // TODO: update ticks, fee growth, etc.
    }

    /// Remove liquidity from a position
    pub fn burn(&mut self, owner: Address, tick_lower: i32, tick_upper: i32, amount: U256) {
        if let Some(position) = self.positions.get_mut(&(owner, tick_lower, tick_upper)) {
            position.liquidity -= I256::try_from(amount).unwrap_or(I256::ZERO);
            self.liquidity -= amount;
            // TODO: update ticks, fee growth, etc.
        }
    }

    /// Collect fees from a position
    pub fn collect(
        &mut self,
        owner: Address,
        tick_lower: i32,
        tick_upper: i32,
        amount0_requested: U256,
        amount1_requested: U256,
    ) -> (U256, U256) {
        if let Some(position) = self.positions.get_mut(&(owner, tick_lower, tick_upper)) {
            let amount0 = amount0_requested.min(position.tokens_owed0);
            let amount1 = amount1_requested.min(position.tokens_owed1);
            position.tokens_owed0 -= amount0;
            position.tokens_owed1 -= amount1;
            (amount0, amount1)
        } else {
            (U256::ZERO, U256::ZERO)
        }
    }

    /// Main swap function - closely follows Uniswap V3 swap logic
    pub fn swap(&mut self, params: SwapParams) -> SwapResult {
        assert!(params.amount_specified != I256::ZERO, "AS");
        assert!(self.slot0.unlocked, "LOK");

        let slot0_start = self.slot0.clone();

        let valid_limit = if params.zero_for_one {
            params.sqrt_price_limit_x96 < slot0_start.sqrt_price_x96
                && params.sqrt_price_limit_x96 > MIN_SQRT_RATIO
        } else {
            params.sqrt_price_limit_x96 > slot0_start.sqrt_price_x96
                && params.sqrt_price_limit_x96 < MAX_SQRT_RATIO
        };
        assert!(valid_limit, "SPL");

        self.slot0.unlocked = false;

        let exact_input = params.amount_specified > I256::ZERO;

        // Swap state
        let mut amount_specified_remaining = params.amount_specified;
        let mut amount_calculated = I256::ZERO;
        let mut sqrt_price_x96 = slot0_start.sqrt_price_x96;
        let mut tick = slot0_start.tick;
        let mut liquidity = self.liquidity;

        // Main swap loop (simplified)
        while amount_specified_remaining != I256::ZERO
            && sqrt_price_x96 != params.sqrt_price_limit_x96
        {
            // For demo: just use the edge tick, in a real implementation you'd scan for the next initialized tick
            let next_tick = if params.zero_for_one {
                MIN_TICK
            } else {
                MAX_TICK
            };
            let sqrt_price_next_x96 = get_sqrt_ratio_at_tick(next_tick);

            let target_price_x96 = if (params.zero_for_one
                && sqrt_price_next_x96 < params.sqrt_price_limit_x96)
                || (!params.zero_for_one && sqrt_price_next_x96 > params.sqrt_price_limit_x96)
            {
                params.sqrt_price_limit_x96
            } else {
                sqrt_price_next_x96
            };

            let (new_sqrt_price_x96, amount_in, amount_out, fee_amount) = compute_swap_step(
                sqrt_price_x96,
                target_price_x96,
                liquidity,
                amount_specified_remaining,
                self.fee,
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
            // TODO: Tick crossing and state updates

            // For demonstration, break after one loop
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

// Example usage:
fn main() {
    let factory = Address::ZERO;
    let token0 = Address::repeat(0x01);
    let token1 = Address::repeat(0x02);
    let slot0 = Slot0 {
        sqrt_price_x96: U256::from(79228162514264337593543950336u128),
        tick: 0,
        observation_index: 0,
        observation_cardinality: 0,
        observation_cardinality_next: 0,
        fee_protocol: 0,
        unlocked: true,
    };
    let mut pool = UniswapV3Pool::new(
        factory,
        token0,
        token1,
        3000,
        60,
        U256::from(1_000_000u64),
        slot0,
    );

    let user = Address::repeat(0x09);

    pool.mint(user, -120, 120, U256::from(10_000u64));
    println!("Pool after mint: {:?}", pool);

    let params = SwapParams {
        recipient: user,
        zero_for_one: true,
        amount_specified: I256::from(1_000_000u64),
        sqrt_price_limit_x96: U256::from_dec_str("79228162514264337593543950336").unwrap(),
    };
    let output = pool.swap(params);

    println!("amount0: {}, amount1: {}", output.amount0, output.amount1);
}
