use crate::state::bonding_curve::*;
use crate::util::bps_mul;
use crate::{errors::ContractError, util::BASIS_POINTS_DIVISOR};
use allocation::AllocationData;
use anchor_lang::prelude::*;
use segment::*;
use std::fmt::{self};
use std::ops::{Add, Div, Mul};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, Debug, PartialEq, Default)]
pub enum BondingCurveStatus {
    #[default]
    Inactive,
    Prepared,
    Active,
    Complete,
    Launched,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, Debug, PartialEq, Default)]
pub struct BondingCurveSupplyAllocation {
    pub creator_vested_supply: u64,
    pub presale_supply: u64,
    pub bonding_supply: u64,
    pub pool_supply: u64,
    pub cex_supply: u64,
    pub launch_brandkit_supply: u64,
    pub lifetime_brandkit_supply: u64,
    pub platform_supply: u64,
}
impl BondingCurveSupplyAllocation {
    pub fn new_from_allocation(allocation: &AllocationData, token_total_supply: u64) -> Self {
        let creator_vested_supply = bps_mul(allocation.creator, token_total_supply).unwrap();
        let presale_supply = bps_mul(allocation.presale, token_total_supply).unwrap();
        let bonding_supply = bps_mul(allocation.curve_reserve, token_total_supply).unwrap();
        let pool_supply = bps_mul(allocation.pool_reserve, token_total_supply).unwrap();
        let cex_supply = bps_mul(allocation.cex, token_total_supply).unwrap();
        let launch_brandkit_supply =
            bps_mul(allocation.launch_brandkit, token_total_supply).unwrap();
        let lifetime_brandkit_supply =
            bps_mul(allocation.lifetime_brandkit, token_total_supply).unwrap();
        let platform_supply = bps_mul(allocation.platform, token_total_supply).unwrap();
        BondingCurveSupplyAllocation {
            creator_vested_supply,
            presale_supply,
            bonding_supply,
            pool_supply,
            cex_supply,
            launch_brandkit_supply,
            lifetime_brandkit_supply,
            platform_supply,
        }
    }
}

#[account]
#[derive(InitSpace, Debug, Default)]
pub struct BondingCurve {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub cex_authority: Pubkey,
    pub brand_authority: Pubkey,
    pub status: BondingCurveStatus,

    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,

    pub token_total_supply: u64,

    pub sol_launch_threshold: u64,

    pub start_time: i64,
    pub vesting_terms: VestingTerms,

    pub allocation: AllocationData,
    pub supply_allocation: BondingCurveSupplyAllocation,
    #[max_len(16)]
    pub curve_segments: Vec<CurveSegment>,
    // pub current_segment: u8,
    pub bump: u8,
}

impl BondingCurve {
    pub const SEED_PREFIX: &'static str = "bonding-curve";

    pub fn get_signer<'a>(bump: &'a u8, mint: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }

    pub fn create_from_params(
        mint: Pubkey,
        creator: Pubkey,
        brand_authority: Pubkey,
        cex_authority: Pubkey,
        params: &CreateBondingCurveParams,
        clock: &Clock,
        bump: u8,
    ) -> Self {
        let start_time = params.start_time.unwrap_or(clock.unix_timestamp);
        let token_total_supply = params.token_total_supply;

        let allocation: AllocationData = params.allocation.into();

        let supply_allocation =
            BondingCurveSupplyAllocation::new_from_allocation(&allocation, token_total_supply);

        let real_token_reserves = supply_allocation.bonding_supply;

        let real_sol_reserves = 0;
        let sol_launch_threshold = params.sol_launch_threshold;
        let vesting_terms = params.vesting_terms.clone().unwrap_or_default();

        if !params.curve_segments.is_valid() {
            msg!("Invalid Curve Segments");
            panic!("Invalid Curve Segments");
        }

        BondingCurve {
            mint,
            creator,
            brand_authority,
            cex_authority,
            status: BondingCurveStatus::default(),
            real_sol_reserves,
            real_token_reserves,
            token_total_supply,
            supply_allocation,
            sol_launch_threshold,
            start_time,
            allocation,
            curve_segments: params
                .curve_segments
                .into_segment_data(supply_allocation.bonding_supply),
            bump,
            vesting_terms,
        }
    }

    pub fn get_max_attainable_sol(&self) -> Option<u64> {
        let tokens_available = self.real_token_reserves;
        if tokens_available == 0 {
            return Some(self.real_sol_reserves);
        }
        self.get_buy_price(tokens_available)
    }

    pub fn is_started(&self, clock: &Clock) -> bool {
        let now = clock.unix_timestamp;
        now >= self.start_time
    }

    pub fn msg(&self) -> () {
        msg!("{:#?}", self);
    }

    pub fn invariant<'info>(
        bonding_curve: &Box<Account<'info, BondingCurve>>,
        ctx: &mut BondingCurveLockerCtx<'info>,
    ) -> Result<()> {
        let tkn_account = &mut ctx.bonding_curve_account;
        if tkn_account.owner != ctx.bonding_curve.key() {
            msg!("Invariant failed: invalid token acc supplied");
            return Err(ContractError::BondingCurveInvariant.into());
        }
        tkn_account.reload()?;

        let lamports = bonding_curve.get_lamports();
        let tkn_balance = tkn_account.amount;

        let rent_exemption_balance: u64 =
            Rent::get()?.minimum_balance(8 + BondingCurve::INIT_SPACE as usize);
        let bonding_curve_pool_lamports: u64 = lamports - rent_exemption_balance;

        // Ensure real sol reserves are equal to bonding curve pool lamports
        if bonding_curve_pool_lamports != bonding_curve.real_sol_reserves {
            msg!(
                "real_sol_r:{}, bonding_lamps:{}",
                bonding_curve.real_sol_reserves,
                bonding_curve_pool_lamports
            );
            msg!("Invariant failed: real_sol_reserves != bonding_curve_pool_lamports");
            return Err(ContractError::BondingCurveInvariant.into());
        }

        // Ensure the token total supply is consistent with the reserves
        if bonding_curve.real_token_reserves != tkn_balance {
            msg!("Invariant failed: real_token_reserves != tkn_balance");
            msg!("real_token_reserves: {}", bonding_curve.real_token_reserves);
            msg!("tkn_balance: {}", tkn_balance);
            return Err(ContractError::BondingCurveInvariant.into());
        }

        // Ensure the bonding curve is complete only if real token reserves are zero
        if bonding_curve.status == BondingCurveStatus::Complete
            && bonding_curve.real_token_reserves != 0
        {
            msg!("Invariant failed: bonding curve marked as complete but real_token_reserves != 0");
            return Err(ContractError::BondingCurveInvariant.into());
        }

        let bonding_curve_balance = bonding_curve.get_lamports();
        let min_lamports = Rent::get()?.minimum_balance(8 + BondingCurve::INIT_SPACE as usize);
        let bonding_curve_lamports = bonding_curve_balance - min_lamports;

        // ensure bonding_curve_lamorts is consistent with the reserves
        if bonding_curve.real_sol_reserves != bonding_curve_lamports {
            msg!("Invariant failed: real_sol_reserves != bonding_curve_lamports");
            msg!("real_sol_reserves: {}", bonding_curve.real_sol_reserves);
            msg!("bonding_curve_lamports: {}", bonding_curve_lamports);
            return Err(ContractError::BondingCurveInvariant.into());
        }

        // ensure its complete only is balance is over threshold
        if bonding_curve.status == BondingCurveStatus::Complete
            && bonding_curve_lamports < bonding_curve.sol_launch_threshold
        {
            msg!("bonding_curve_balance: {}", bonding_curve_balance);
            msg!("min_lamports: {}", min_lamports);
            msg!("bonding curve lamports: {}", bonding_curve_lamports);
            msg!(
                "sol_launch_threshold: {}",
                bonding_curve.sol_launch_threshold
            );

            msg!("Invariant failed: bonding curve marked as complete but balance is less than sol_launch_threshold");
            return Err(ContractError::BondingCurveInvariant.into());
        }

        if bonding_curve.status != BondingCurveStatus::Launched && !tkn_account.is_frozen() {
            msg!("Not Launched BondingCurve TokenAccount must always be frozen at the end");
            return Err(ContractError::BondingCurveInvariant.into());
        }
        Ok(())
    }

    pub fn apply_buy(&mut self, sol_amount: u64) -> Option<BuyResult> {
        let tokens_to_send = self.get_tokens_for_buy_sol(sol_amount)?;
        self.real_token_reserves = self.real_token_reserves.checked_sub(tokens_to_send)?;
        self.real_sol_reserves = self.real_sol_reserves.checked_add(sol_amount)?;

        Some(BuyResult {
            token_amount: tokens_to_send,
            sol_amount,
        })
    }

    pub fn apply_sell(&mut self, token_amount: u64) -> Option<SellResult> {
        let new_reserves = self.real_token_reserves.checked_add(token_amount)?;
        if new_reserves > self.supply_allocation.bonding_supply {
            return None;
        }
        let sol_amount = self.get_sell_price(token_amount)?;

        self.real_token_reserves = new_reserves;
        self.real_sol_reserves = self.real_sol_reserves.checked_sub(sol_amount)?;

        Some(SellResult {
            token_amount,
            sol_amount,
        })
    }

    pub fn get_buy_price(&self, tokens: u64) -> Option<u64> {
        if tokens == 0 || tokens > self.real_token_reserves {
            return None;
        }

        let mut remaining_tokens = tokens;
        let mut total_price = 0u64;
        let mut current_supply = self.real_token_reserves;
        println!("segments: {:?}", self.curve_segments);
        for segment in self.curve_segments.iter().rev() {
            if current_supply > segment.end_supply {
                continue;
            }
            println!("current_supply: {}", current_supply);
            let segment_tokens = (current_supply - segment.start_supply).min(remaining_tokens);
            println!("segment_tokens: {}", segment_tokens);
            let segment_price = calculate_segment_price(segment, current_supply, segment_tokens)?;
            println!("segment_price: {}", segment_price);
            total_price = total_price.checked_add(segment_price)?;
            println!("total_price: {}", total_price);
            remaining_tokens = remaining_tokens.checked_sub(segment_tokens)?;
            current_supply = current_supply.checked_add(segment_tokens)?;
            println!("remaining_tokens: {}", remaining_tokens);
            println!("current_supply: {}", current_supply);
            if remaining_tokens == 0 {
                break;
            }
        }
        println!("total_price: {}", total_price);

        Some(total_price)
    }

    pub fn get_sell_price(&self, tokens: u64) -> Option<u64> {
        if tokens == 0 {
            return None;
        }

        let mut remaining_tokens = tokens;
        let mut total_price = 0u64;
        let mut current_supply = self.real_token_reserves;

        for segment in self.curve_segments.iter().rev() {
            if current_supply <= segment.start_supply {
                continue;
            }

            let segment_tokens = (current_supply - segment.start_supply).min(remaining_tokens);
            let segment_price =
                calculate_segment_price(segment, current_supply - segment_tokens, segment_tokens)?;

            total_price = total_price.checked_add(segment_price)?;
            remaining_tokens = remaining_tokens.checked_sub(segment_tokens)?;
            current_supply = current_supply.checked_sub(segment_tokens)?;

            if remaining_tokens == 0 {
                break;
            }
        }

        if total_price > self.real_sol_reserves {
            Some(self.real_sol_reserves)
        } else {
            Some(total_price)
        }
    }

    pub fn get_tokens_for_buy_sol(&self, sol_amount: u64) -> Option<u64> {
        println!("get_tokens_for_buy_sol:sol_amount: {}", sol_amount);
        if sol_amount == 0 {
            return None;
        }

        let mut remaining_sol = sol_amount;
        let mut total_tokens = 0u64;
        let mut current_supply = self.real_token_reserves;

        for segment in self.curve_segments.iter().rev() {
            println!(
                "get_tokens_for_buy_sol:segment:remaining_sol: {}",
                remaining_sol
            );
            if current_supply > segment.end_supply {
                println!(
                    "current_supply({}) > segment.end_supply({})",
                    current_supply, segment.end_supply
                );
                continue;
            }

            let segment_sol = calculate_segment_price(
                segment,
                current_supply,
                current_supply - segment.start_supply,
            )?;
            println!("get_tokens_for_buy_sol:segment_sol: {}", segment_sol);
            let segment_tokens = if segment_sol <= remaining_sol {
                remaining_sol = remaining_sol.checked_sub(segment_sol)?;
                current_supply - segment.start_supply
            } else {
                calculate_tokens_for_buy_segment(segment, current_supply, remaining_sol)?
            };
            println!("get_tokens_for_buy_sol:segment_tokens: {}", segment_tokens);

            total_tokens = total_tokens.checked_add(segment_tokens)?;
            current_supply = current_supply.checked_add(segment_tokens)?;

            if remaining_sol == 0 {
                break;
            }
        }
        println!("get_tokens_for_buy_sol:total_tokens: {}", total_tokens);
        Some(total_tokens)
    }

    pub fn get_tokens_for_sell_sol(&self, sol_amount: u64) -> Option<u64> {
        if sol_amount == 0 || sol_amount > self.real_sol_reserves {
            return None;
        }

        let mut remaining_sol = sol_amount;
        let mut total_tokens = 0u64;
        let mut current_supply = self.real_token_reserves;

        for segment in self.curve_segments.iter().rev() {
            if current_supply <= segment.start_supply {
                continue;
            }

            let segment_sol = calculate_segment_price(
                segment,
                segment.start_supply,
                current_supply - segment.start_supply,
            )?;
            println!("segment_sol: {}", segment_sol);
            let segment_tokens = if segment_sol <= remaining_sol {
                remaining_sol = remaining_sol.checked_sub(segment_sol)?;
                current_supply - segment.start_supply
            } else {
                calculate_tokens_for_sell_segment(segment, current_supply, remaining_sol)?
            };

            total_tokens = total_tokens.checked_add(segment_tokens)?;
            current_supply = current_supply.checked_sub(segment_tokens)?;

            if remaining_sol == 0 {
                break;
            }
        }

        Some(total_tokens)
    }
}

impl fmt::Display for BondingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BondingCurve {{ creator: {:?}, real_sol_reserves: {:?}, real_token_reserves: {:?}, token_total_supply: {:?}, supply_allocation: {:?}, sol_launch_threshold: {:?}, start_time: {:?}, status: {:?}, allocation: \n{:?} \n}}",
            self.creator,
            self.real_sol_reserves,
            self.real_token_reserves, self.token_total_supply, self.supply_allocation, self.sol_launch_threshold, self.start_time, self.status,
            self.allocation
        )
    }
}

fn calculate_linear_price(
    slope: u64,
    intercept: u64,
    tokens: u64,
    start_supply: u64,
) -> Option<u64> {
    // let slope = (slope).div(BASIS_POINTS_DIVISOR);
    // let intercept = (intercept).div(BASIS_POINTS_DIVISOR);
    println!(
        "slope: {}, intercept: {}, tokens: {}, start_supply: {}, slope: {}, intercept: {}",
        slope, intercept, tokens, start_supply, slope, intercept
    );

    let part1 = start_supply;
    println!("part1 (start_supply as u128): {}", part1);

    let part2 = part1.mul(slope);
    println!("part2 (start_supply * slope): {}", part2);

    let part3 = part2.add(intercept);
    println!("part3 (part2 + intercept): {}", part3);

    let part4 = (tokens).mul(slope);
    println!("part4 (tokens * slope): {}", part4);

    let part5 = part4.div(2);
    println!("part5 (part4 / 2): {}", part5);

    let part6 = part3.add(part5);
    println!("part6 (part3 + part5): {}", part6);

    let part7 = part6.mul(tokens);
    println!("part7 (part6 * tokens): {}", part7);

    let result = part7.div(10000);
    println!("result: {}", result);

    Some(result)
}

// TODO TEST
fn calculate_exponential_price(base: u64, exponent: u32, scale: u64, tokens: u64) -> Option<u64> {
    // println!("scale: {}", scale);
    println!(
        "base: {}, exponent: {}, scale: {}, tokens: {}",
        base, exponent, scale, tokens
    );

    let powed = base.pow(exponent);
    println!("powed (base^exponent): {}", powed);

    let tokens_price = powed as u128 * tokens as u128;
    println!("tokens_price (powed * tokens): {}", tokens_price);

    let scaled = tokens_price / scale as u128;
    println!("scaled (tokens_price / scale): {}", scaled);

    let result = scaled;
    println!("result: {}", result);

    if result > u64::MAX as u128 {
        println!("result > u64::MAX, {}", result);
        return None;
    }
    Some(result as u64)
}

pub fn calculate_segment_price(
    segment: &CurveSegment,
    start_supply: u64,
    tokens: u64,
) -> Option<u64> {
    println!(
        "calculate_segment_price:SegmentType: {:?}, start_supply: {}, tokens: {}",
        segment.segment_type, start_supply, tokens
    );

    match segment.segment_type {
        SegmentType::Constant(price) => {
            println!("Constant price: {}", price);
            Some(price * tokens)
        }
        SegmentType::Linear(slope, intercept) => {
            calculate_linear_price(slope, intercept, tokens, start_supply)
        }
        SegmentType::Exponential(base, exponent, scale) => {
            calculate_exponential_price(base, exponent, scale, tokens)
        }
    }
}

pub fn calculate_tokens_for_segment(
    segment: &CurveSegment,
    start_supply: u64,
    sol_amount: u64,
    is_buy: bool,
) -> Option<u64> {
    println!(
        "SegmentType: {:?}, start_supply: {}, sol_amount: {}, is_buy: {}",
        segment.segment_type, start_supply, sol_amount, is_buy
    );

    match segment.segment_type {
        SegmentType::Constant(price) => {
            println!("Constant price: {}", price);
            Some(sol_amount / price)
        }
        SegmentType::Linear(slope, intercept) => {
            let a = if is_buy { 20000 } else { 20000 };
            println!("a: {}", a);

            let part1 = sol_amount as u128;
            println!("part1 (amount as u128): {}", part1);

            let part2 = part1.checked_mul(a)?;
            println!("part2 (part1 * a): {}", part2);

            let part3 = part2.checked_div(slope as u128)?;
            println!("part3 (part2 / slope): {}", part3);

            let part4 = part3.checked_sub(intercept as u128)?;
            println!("part4 (part3 - intercept): {}", part4);

            let part5 = part4.checked_sub(start_supply as u128 * 2)?;
            println!("part5 (part4 - start_supply * 2): {}", part5);

            let part6 = part5.checked_add(1)?;
            println!("part6 (part5 + 1): {}", part6);

            let part7 = part6.pow(1 / 2);
            println!("part7 (part6^0.5): {}", part7);

            let part8 = part7.checked_sub(1)?;
            println!("part8 (part7 - 1): {}", part8);

            let result = part8.checked_div(2)? as u64;
            println!("result: {}", result);

            Some(result)
        }
        SegmentType::Exponential(base, exponent, scale) => {
            if is_buy {
                // Solve the equation: sol_amount = base^exponent * tokens / scale
                // Rearranging to solve for tokens:
                // tokens = sol_amount * scale / (base^exponent)
                let powered = (base as u128).pow(exponent);
                println!("powered (base^exponent): {}", powered);

                let tokens = sol_amount as u128 * scale as u128 / powered;
                println!("tokens: {}", tokens as u64);

                return Some(tokens as u64);
            } else {
                // Solve the equation: sol_amount = base^exponent * tokens / scale
                // Rearranging to solve for tokens:
                // tokens = sol_amount * scale / (base^exponent)
                let powered = (base as u128).pow(exponent);
                println!("powered (base^exponent): {}", powered);

                let tokens = sol_amount as u128 * scale as u128 / powered;
                println!("tokens: {}", tokens as u64);

                return Some(tokens as u64);
            };
        }
    }
}

pub fn calculate_tokens_for_buy_segment(
    segment: &CurveSegment,
    start_supply: u64,
    sol_amount: u64,
) -> Option<u64> {
    println!(
        "Buy segment, start_supply: {}, sol_amount: {}",
        start_supply, sol_amount
    );
    calculate_tokens_for_segment(segment, start_supply, sol_amount, true)
}

pub fn calculate_tokens_for_sell_segment(
    segment: &CurveSegment,
    end_supply: u64,
    sol_amount: u64,
) -> Option<u64> {
    println!(
        "Sell segment, end_supply: {}, sol_amount: {}",
        end_supply, sol_amount
    );
    calculate_tokens_for_segment(segment, end_supply, sol_amount, false)
}
