use crate::errors::ContractError;
use crate::state::allocation::AllocationData;
use crate::state::bonding_curve::*;
use crate::util::{bps_mul, bps_mul_raw, BASIS_POINTS_DIVISOR};
use anchor_lang::prelude::*;
use std::fmt::{self};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, Debug, PartialEq, Default)]
pub enum BondingCurveStatus {
    #[default]
    Inactive,
    Prepared,
    Active,
    Complete,
    Launched,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Debug)]
pub enum CurveType {
    Constant,
    Linear,
    Exponential,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Debug)]
pub struct CurveSegmentDef {
    pub curve_type: CurveType,
    pub start_supply_bps: u64,
    pub end_supply_bps: u64,
    pub params: [u64; 3], // Parameters for the curve (meaning depends on curve type)
}

use std::vec::Vec;
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Debug)]
pub struct CurveSegment {
    pub curve_type: CurveType,
    pub start_supply: u64,
    pub end_supply: u64,
    pub params: [u64; 3], // Parameters for the curve (meaning depends on curve type)
}
pub trait CurveSegmentInput {
    fn is_valid(&self) -> bool;
    fn into_segment_data(&self, bonding_supply: u64) -> Vec<CurveSegment>;
}
impl CurveSegmentInput for Vec<CurveSegmentDef> {
    fn is_valid(&self) -> bool {
        // Must check that all end and supply bps are in consecutive order,
        // all add up to 10_000, there's no overlap, and no gaps
        // First start supply must be 0
        self.first().map(|segment| segment.start_supply_bps == 0).unwrap_or(false)
            && // Last end supply must be 10_000
            self.last().map(|segment| segment.end_supply_bps == BASIS_POINTS_DIVISOR).unwrap_or(false)
            && // All segments must be valid
            self.iter().all(|segment| {
                segment.start_supply_bps <= segment.end_supply_bps
                    && segment.start_supply_bps < BASIS_POINTS_DIVISOR
                    && segment.end_supply_bps <= BASIS_POINTS_DIVISOR
            })
            && // No overlapping segments
            self.iter().enumerate().all(|(i, segment)| {
                let next_segment = self.get(i + 1);
                if let Some(next_segment) = next_segment {
                    segment.end_supply_bps == next_segment.start_supply_bps
                } else {
                    true
                }
            })
    }

    fn into_segment_data(&self, bonding_supply: u64) -> Vec<CurveSegment> {
        // Map each segment def basis points to actual tokens
        let mut segments = Vec::with_capacity(self.len());
        for segment in self.iter() {
            let start_supply = bps_mul(segment.start_supply_bps, bonding_supply).unwrap();
            let end_supply = bps_mul(segment.end_supply_bps, bonding_supply).unwrap();
            segments.push(CurveSegment {
                curve_type: segment.curve_type.clone(),
                start_supply,
                end_supply,
                params: segment.params,
            });
        }
        segments
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
    pub virtual_token_multiplier_bps: u64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u128,
    pub initial_virtual_token_reserves: u128,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub creator_vested_supply: u64,
    pub presale_supply: u64,
    pub bonding_supply: u64,
    pub pool_supply: u64,
    pub cex_supply: u64,
    pub launch_brandkit_supply: u64,
    pub lifetime_brandkit_supply: u64,
    pub platform_supply: u64,
    pub sol_launch_threshold: u64,
    pub start_time: i64,
    pub vesting_terms: VestingTerms,
    pub allocation: AllocationData,
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
        let virtual_sol_reserves = params.virtual_sol_reserves;
        let virtual_token_multiplier = params.virtual_token_multiplier_bps;
        let allocation: AllocationData = params.allocation.into();

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

        let real_token_reserves = bonding_supply;
        let virtual_token_reserves = bonding_supply as u128
            + bps_mul_raw(params.virtual_token_multiplier_bps, bonding_supply).unwrap();

        let initial_virtual_token_reserves = virtual_token_reserves;
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
            initial_virtual_token_reserves,
            virtual_token_multiplier_bps: virtual_token_multiplier,
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            token_total_supply,
            bonding_supply,
            pool_supply,
            creator_vested_supply,
            presale_supply,
            cex_supply,
            launch_brandkit_supply,
            lifetime_brandkit_supply,
            platform_supply,
            sol_launch_threshold,
            start_time,
            allocation,
            curve_segments: params.curve_segments.into_segment_data(bonding_supply),
            // current_segment: 0,
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

    pub fn get_buy_price(&self, tokens: u64) -> Option<u64> {
        if tokens == 0 || tokens > self.real_token_reserves {
            return None;
        }

        let mut remaining_tokens = tokens;
        let mut total_price = 0u64;
        let mut current_supply = self.real_token_reserves;

        for segment in &self.curve_segments {
            if current_supply >= segment.end_supply {
                continue;
            }

            let segment_tokens = (segment.end_supply - current_supply).min(remaining_tokens);
            let segment_price =
                self.calculate_segment_price(segment, current_supply, segment_tokens)?;

            total_price = total_price.checked_add(segment_price)?;
            remaining_tokens = remaining_tokens.checked_sub(segment_tokens)?;
            current_supply = current_supply.checked_add(segment_tokens)?;

            if remaining_tokens == 0 {
                break;
            }
        }

        Some(total_price)
    }

    fn calculate_segment_price(
        &self,
        segment: &CurveSegment,
        start_supply: u64,
        tokens: u64,
    ) -> Option<u64> {
        match segment.curve_type {
            CurveType::Constant => Some(segment.params[0].checked_mul(tokens)?),
            CurveType::Linear => {
                let slope = segment.params[0];
                let intercept = segment.params[1];
                Some(
                    ((start_supply as u128)
                        .checked_mul(slope as u128)?
                        .checked_add(intercept as u128)?
                        .checked_add(
                            (tokens as u128)
                                .checked_mul(slope as u128)?
                                .checked_div(2)?,
                        )?
                        .checked_mul(tokens as u128)?)
                    .checked_div(10000)? as u64,
                )
            }
            CurveType::Exponential => {
                let base = segment.params[0];
                let exponent = segment.params[1];
                let scale = segment.params[2];
                println!("calculate_segment_price:base: {}", base);
                println!("calculate_segment_price:exponent: {}", exponent);
                println!("calculate_segment_price:scale: {}", scale);
                println!("calculate_segment_price:tokens: {}", tokens);
                Some(
                    ((base as u128)
                        .pow(exponent as u32)
                        .checked_mul(tokens as u128)?
                        .checked_div(scale as u128)?) as u64,
                )
            }
        }
    }

    pub fn apply_buy(&mut self, sol_amount: u64) -> Option<BuyResult> {
        let tokens_to_send = self.get_tokens_for_buy_sol(sol_amount)?;
        println!("apply_buy:tokens_received: {}", tokens_to_send);
        println!(
            "apply_buy:virtual_token_reserves: {}",
            self.virtual_token_reserves
        );
        self.virtual_token_reserves = self
            .virtual_token_reserves
            .checked_sub(tokens_to_send as u128)?;
        println!("1");
        self.real_token_reserves = self.real_token_reserves.checked_sub(tokens_to_send)?;
        println!("2");
        self.virtual_sol_reserves = self.virtual_sol_reserves.checked_add(sol_amount)?;
        println!("3");
        self.real_sol_reserves = self.real_sol_reserves.checked_add(sol_amount)?;
        println!("4");
        // self.update_current_segment();

        Some(BuyResult {
            token_amount: tokens_to_send,
            sol_amount,
        })
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
            let segment_price = self.calculate_segment_price(
                segment,
                current_supply - segment_tokens,
                segment_tokens,
            )?;

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

    pub fn apply_sell(&mut self, token_amount: u64) -> Option<SellResult> {
        let sol_amount = self.get_sell_price(token_amount)?;

        self.virtual_token_reserves = self
            .virtual_token_reserves
            .checked_add(token_amount as u128)?;
        self.real_token_reserves = self.real_token_reserves.checked_add(token_amount)?;
        self.virtual_sol_reserves = self.virtual_sol_reserves.checked_sub(sol_amount)?;
        self.real_sol_reserves = self.real_sol_reserves.checked_sub(sol_amount)?;

        // self.update_current_segment();

        Some(SellResult {
            token_amount,
            sol_amount,
        })
    }

    pub fn get_tokens_for_buy_sol(&self, sol_amount: u64) -> Option<u64> {
        if sol_amount == 0 {
            return None;
        }

        let mut remaining_sol = sol_amount;
        let mut total_tokens = 0u64;
        let mut current_supply = self.real_token_reserves;

        println!("get_t_bs:remaining_sol: {}", remaining_sol);
        println!("get_t_bs:total_tokens: {}", total_tokens);
        println!("get_t_bs:current_supply: {}", current_supply);

        for segment in &self.curve_segments {
            println!("get_t_bs:segment: {:?}", segment);
            if current_supply > segment.end_supply {
                continue;
            }

            let segment_sol = self.calculate_segment_price(
                segment,
                current_supply,
                current_supply - segment.start_supply,
            )?;
            println!("get_t_bs:segment_sol: {}", segment_sol);
            let segment_tokens = if segment_sol <= remaining_sol {
                remaining_sol = remaining_sol.checked_sub(segment_sol)?;
                segment.end_supply - current_supply
            } else {
                self.calculate_tokens_for_buy_segment(segment, current_supply, remaining_sol)?
            };
            println!("get_t_bs:segment_tokens: {}", segment_tokens);

            total_tokens = total_tokens.checked_add(segment_tokens)?;
            current_supply = current_supply.checked_add(segment_tokens)?;

            if remaining_sol == 0 {
                break;
            }
        }

        Some(total_tokens)
    }

    fn calculate_tokens_for_buy_segment(
        &self,
        segment: &CurveSegment,
        start_supply: u64,
        sol_amount: u64,
    ) -> Option<u64> {
        match segment.curve_type {
            CurveType::Constant => Some(sol_amount.checked_div(segment.params[0])?),
            CurveType::Linear => {
                let slope = segment.params[0];
                let intercept = segment.params[1];
                Some(
                    ((sol_amount as u128)
                        .checked_mul(20000)?
                        .checked_div(slope as u128)?
                        .checked_sub(intercept as u128)?
                        .checked_sub(start_supply as u128 * 2)?
                        .checked_add(1)?
                        .pow(1 / 2)
                        .checked_sub(1)?
                        .checked_div(2)?) as u64,
                )
            }
            CurveType::Exponential => {
                let base = segment.params[0];
                let exponent = segment.params[1];
                let scale = segment.params[2];

                if base == 0 || exponent == 0 || scale == 0 {
                    return None;
                }

                let scaled_sol = (sol_amount as u128).checked_mul(scale as u128)?;
                println!("calculate_tokens_for_buy_segment:base: {}", base);
                println!("calculate_tokens_for_buy_segment:exponent: {}", exponent);
                println!("calculate_tokens_for_buy_segment:scale: {}", scale);
                println!(
                    "calculate_tokens_for_buy_segment:sol_amount: {}",
                    sol_amount
                );
                println!(
                    "calculate_tokens_for_buy_segment:scaled_sol: {}",
                    scaled_sol
                );
                let powered: u128 = base.checked_pow(exponent as u32)?.into();
                println!("calculate_tokens_for_buy_segment:powered: {}", powered);
                let tokens = powered.checked_div(scaled_sol)?;
                println!("calculate_tokens_for_buy_segment:tokens: {}", tokens);
                let tokens_u64: u64 = tokens.try_into().ok()?;
                Some(tokens_u64)
            }
        }
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

            let segment_sol = self.calculate_segment_price(
                segment,
                segment.start_supply,
                current_supply - segment.start_supply,
            )?;
            let segment_tokens = if segment_sol <= remaining_sol {
                remaining_sol = remaining_sol.checked_sub(segment_sol)?;
                current_supply - segment.start_supply
            } else {
                self.calculate_tokens_for_sell_segment(segment, current_supply, remaining_sol)?
            };

            total_tokens = total_tokens.checked_add(segment_tokens)?;
            current_supply = current_supply.checked_sub(segment_tokens)?;

            if remaining_sol == 0 {
                break;
            }
        }

        Some(total_tokens)
    }

    fn calculate_tokens_for_sell_segment(
        &self,
        segment: &CurveSegment,
        end_supply: u64,
        sol_amount: u64,
    ) -> Option<u64> {
        match segment.curve_type {
            CurveType::Constant => Some(sol_amount.checked_div(segment.params[0])?),
            CurveType::Linear => {
                let slope = segment.params[0];
                let intercept = segment.params[1];
                Some(
                    ((sol_amount as u128)
                        .checked_mul(20000)?
                        .checked_div(slope as u128)?
                        .checked_add(intercept as u128)?
                        .checked_add(end_supply as u128 * 2)?
                        .checked_add(1)?
                        .pow(1 / 2)
                        .checked_sub(1)?
                        .checked_div(2)?) as u64,
                )
            }
            CurveType::Exponential => {
                let base = segment.params[0];
                let exponent = segment.params[1];
                let scale = segment.params[2];
                Some(
                    ((sol_amount as u128)
                        .checked_mul(scale as u128)?
                        .pow(1 / 2)
                        .checked_div((base as u128).pow(exponent as u32 / 2))?)
                        as u64,
                )
            }
        }
    }

    // pub fn update_current_segment(&mut self) {
    //     self.current_segment = self
    //         .curve_segments
    //         .iter()
    //         .position(|segment| self.real_token_reserves < segment.end_supply)
    //         .unwrap_or(self.curve_segments.len() - 1) as u8;
    // }

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

        // Ensure the virtual reserves are always positive
        if bonding_curve.virtual_sol_reserves <= 0 {
            msg!("Invariant failed: virtual_sol_reserves <= 0");
            return Err(ContractError::BondingCurveInvariant.into());
        }
        if bonding_curve.virtual_token_reserves <= 0 {
            msg!("Invariant failed: virtual_token_reserves <= 0");
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
}

impl fmt::Display for BondingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BondingCurve {{ creator: {:?}, initial_virtual_token_reserves: {:?}, virtual_sol_reserves: {:?}, virtual_token_reserves: {:?}, real_sol_reserves: {:?}, real_token_reserves: {:?}, token_total_supply: {:?}, presale_supply: {:?}, bonding_supply: {:?}, sol_launch_threshold: {:?}, start_time: {:?}, status: {:?}, allocation: \n{:?} \n}}",
            self.creator,
            self.initial_virtual_token_reserves,
            self.virtual_sol_reserves, self.virtual_token_reserves, self.real_sol_reserves,
            self.real_token_reserves, self.token_total_supply, self.presale_supply,
            self.bonding_supply, self.sol_launch_threshold, self.start_time, self.status,
            self.allocation
        )
    }
}
