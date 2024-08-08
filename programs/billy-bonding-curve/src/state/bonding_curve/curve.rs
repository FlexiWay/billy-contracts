use crate::errors::ContractError;
use crate::state::allocation::AllocationData;
use crate::state::bonding_curve::*;
use crate::util::{bps_mul, bps_mul_raw};
use anchor_lang::prelude::*;
use segment::*;
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

        println!("1");
        self.real_token_reserves = self.real_token_reserves.checked_sub(tokens_to_send)?;
        println!("2");

        self.real_sol_reserves = self.real_sol_reserves.checked_add(sol_amount)?;
        println!("4");

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

        self.real_token_reserves = self.real_token_reserves.checked_add(token_amount)?;
        self.real_sol_reserves = self.real_sol_reserves.checked_sub(sol_amount)?;

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
}

impl fmt::Display for BondingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BondingCurve {{ creator: {:?}, real_sol_reserves: {:?}, real_token_reserves: {:?}, token_total_supply: {:?}, presale_supply: {:?}, bonding_supply: {:?}, sol_launch_threshold: {:?}, start_time: {:?}, status: {:?}, allocation: \n{:?} \n}}",
            self.creator,
            self.real_sol_reserves,
            self.real_token_reserves, self.token_total_supply, self.presale_supply,
            self.bonding_supply, self.sol_launch_threshold, self.start_time, self.status,
            self.allocation
        )
    }
}
