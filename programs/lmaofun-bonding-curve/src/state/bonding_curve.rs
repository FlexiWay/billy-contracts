use crate::errors::ProgramError;
use anchor_lang::prelude::*;
use anchor_lang::Lamports;
use std::fmt::{self};
#[derive(Debug, Clone)]
pub struct BuyResult {
    pub token_amount: u64,
    pub sol_amount: u64,
}

#[derive(Debug, Clone)]
pub struct SellResult {
    pub token_amount: u64,
    pub sol_amount: u64,
}

use super::allocation::AllocationData;

#[account]
#[derive(InitSpace, Debug, Default)]
pub struct BondingCurve {
    pub creator: Pubkey,

    pub virtual_token_multiplier: f64,

    pub virtual_sol_reserves: u64,

    // using u128 to avoid overflow
    pub virtual_token_reserves: u128,
    pub initial_virtual_token_reserves: u128,

    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,

    pub token_total_supply: u64,

    pub presale_supply: u64,
    pub bonding_supply: u64,
    pub cex_supply: u64,
    pub launch_brandkit_supply: u64,
    pub lifetime_brandkit_supply: u64,
    pub platform_supply: u64,

    pub sol_launch_threshold: u64,
    pub start_time: i64,
    pub complete: bool,

    pub allocation: AllocationData,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateBondingCurveParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub start_time: Option<i64>,

    pub token_total_supply: u64,
    pub sol_launch_threshold: u64,

    pub virtual_token_multiplier: f64,
    pub virtual_sol_reserves: u64, // should this be fixed instead?

    pub allocation: AllocationData,
}

impl BondingCurve {
    pub const SEED_PREFIX: &'static str = "bonding-curve";

    pub fn get_signer<'a>(mint: &'a Pubkey, bump: &'a u8) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }

    pub fn update_from_params(
        &mut self,
        creator: Pubkey,
        params: &CreateBondingCurveParams,
        clock: &Clock,
    ) -> &mut Self {
        let start_time = if let Some(start_time) = params.start_time {
            start_time
        } else {
            clock.unix_timestamp
        };
        let token_total_supply = params.token_total_supply;

        let virtual_sol_reserves = params.virtual_sol_reserves;
        let virtual_token_multiplier = params.virtual_token_multiplier;

        let allocation = params.allocation;

        let presale_supply = (token_total_supply as f64 * allocation.presale / 100.0) as u64;
        let bonding_supply = (token_total_supply as f64 * allocation.pool_reserve / 100.0) as u64;
        let cex_supply = (token_total_supply as f64 * allocation.cex / 100.0) as u64;
        let launch_brandkit_supply =
            (token_total_supply as f64 * allocation.launch_brandkit / 100.0) as u64;
        let lifetime_brandkit_supply =
            (token_total_supply as f64 * allocation.lifetime_brandkit / 100.0) as u64;
        let platform_supply = (token_total_supply as f64 * allocation.platform / 100.0) as u64;
        let real_token_reserves = bonding_supply;
        let virtual_token_reserves =
            (bonding_supply as f64 * ((100f64 + virtual_token_multiplier) / 100f64)) as u128;

        let initial_virtual_token_reserves = virtual_token_reserves;

        let real_sol_reserves = 0;
        let sol_launch_threshold = params.sol_launch_threshold;
        let creator = creator;
        let complete = false;

        self.clone_from(&BondingCurve {
            creator,
            initial_virtual_token_reserves,
            virtual_token_multiplier,
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            token_total_supply,

            presale_supply,
            bonding_supply,
            cex_supply,
            launch_brandkit_supply,
            lifetime_brandkit_supply,
            platform_supply,

            sol_launch_threshold,
            start_time,
            complete,
            allocation,
        });
        self
    }

    pub fn get_max_attainable_sol(&self) -> Option<u64> {
        // Calculate the number of tokens available for purchase
        let tokens_available = self.real_token_reserves;

        // If no tokens are available, return the current real SOL reserves
        if tokens_available == 0 {
            return Some(self.real_sol_reserves);
        }

        // Calculate the product of reserves (constant in the bonding curve equation)
        let product_of_reserves =
            (self.virtual_sol_reserves as u128).checked_mul(self.virtual_token_reserves as u128)?;

        // Calculate the new virtual token reserves after all tokens are bought
        let new_virtual_token_reserves = self
            .virtual_token_reserves
            .checked_sub(tokens_available as u128)?;

        // Calculate the new virtual SOL reserves using the constant product formula
        let new_virtual_sol_reserves = product_of_reserves
            .checked_div(new_virtual_token_reserves)?
            .checked_add(1)?;

        // Calculate the difference in virtual SOL reserves
        let sol_increase =
            new_virtual_sol_reserves.checked_sub(self.virtual_sol_reserves as u128)?;

        // Add the increase to the current real SOL reserves
        let max_attainable_sol = (self.real_sol_reserves as u128).checked_add(sol_increase)?;

        // Convert to u64 and return
        max_attainable_sol.try_into().ok()

        // TODO CALCULATE PRESALE SOL VALUE
    }
    pub fn get_buy_price(&self, tokens: u64) -> Option<u64> {
        msg!("get_buy_price: tokens: {}", tokens);
        if tokens == 0 || tokens > self.virtual_token_reserves as u64 {
            return None;
        }

        let product_of_reserves =
            (self.virtual_sol_reserves as u128).checked_mul(self.virtual_token_reserves as u128)?;
        msg!(
            "get_buy_price: product_of_reserves: {}",
            product_of_reserves
        );
        let new_virtual_token_reserves =
            (self.virtual_token_reserves as u128).checked_sub(tokens as u128)?;
        msg!(
            "get_buy_price: new_virtual_token_reserves: {}",
            new_virtual_token_reserves
        );
        let new_virtual_sol_reserves = product_of_reserves
            .checked_div(new_virtual_token_reserves)?
            .checked_add(1)?;
        msg!(
            "get_buy_price: new_virtual_sol_reserves: {}",
            new_virtual_sol_reserves
        );
        let amount_needed =
            new_virtual_sol_reserves.checked_sub(self.virtual_sol_reserves as u128)?;
        msg!("get_buy_price: amount_needed: {}", amount_needed);

        amount_needed.try_into().ok()
    }

    pub fn apply_buy(&mut self, sol_amount: u64) -> Option<BuyResult> {
        msg!("ApplyBuy: sol_amount: {}", sol_amount);

        let final_token_amount = self.get_tokens_for_buy_sol(sol_amount)?;
        msg!("ApplyBuy: final_token_amount: {}", final_token_amount);
        let new_virtual_token_reserves =
            (self.virtual_token_reserves as u128).checked_sub(final_token_amount as u128)?;
        msg!(
            "ApplyBuy: new_virtual_token_reserves: {}",
            new_virtual_token_reserves
        );
        let new_real_token_reserves =
            (self.real_token_reserves as u128).checked_sub(final_token_amount as u128)?;
        msg!(
            "ApplyBuy: new_real_token_reserves: {}",
            new_real_token_reserves
        );

        let new_virtual_sol_reserves =
            (self.virtual_sol_reserves as u128).checked_add(sol_amount as u128)?;
        msg!(
            "ApplyBuy: new_virtual_sol_reserves: {}",
            new_virtual_sol_reserves
        );
        let new_real_sol_reserves =
            (self.real_sol_reserves as u128).checked_add(sol_amount as u128)?;
        msg!("ApplyBuy: new_real_sol_reserves: {}", new_real_sol_reserves);
        self.virtual_token_reserves = new_virtual_token_reserves.try_into().ok()?;
        msg!(
            "ApplyBuy: updated virtual_token_reserves: {}",
            self.virtual_token_reserves
        );
        self.real_token_reserves = new_real_token_reserves.try_into().ok()?;
        msg!(
            "ApplyBuy: updated real_token_reserves: {}",
            self.real_token_reserves
        );
        self.virtual_sol_reserves = new_virtual_sol_reserves.try_into().ok()?;
        msg!(
            "ApplyBuy: updated virtual_sol_reserves: {}",
            self.virtual_sol_reserves
        );
        self.real_sol_reserves = new_real_sol_reserves.try_into().ok()?;
        msg!(
            "ApplyBuy: updated real_sol_reserves: {}",
            self.real_sol_reserves
        );
        self.msg();
        Some(BuyResult {
            token_amount: final_token_amount,
            sol_amount,
        })
    }

    pub fn get_sell_price(&self, tokens: u64) -> Option<u64> {
        msg!("get_sell_price: tokens: {}", tokens);
        if tokens == 0 || tokens > self.virtual_token_reserves as u64 {
            return None;
        }

        let scaling_factor = self.initial_virtual_token_reserves as u128;
        msg!("get_sell_price: scaling_factor: {}", scaling_factor);

        let scaled_tokens = (tokens as u128).checked_mul(scaling_factor)?;
        msg!("get_sell_price: scaled_tokens: {}", scaled_tokens);
        let token_sell_proportion =
            scaled_tokens.checked_div(self.virtual_token_reserves as u128)?;
        msg!(
            "get_sell_price: token_sell_proportion: {}",
            token_sell_proportion
        );
        let sol_received = ((self.virtual_sol_reserves as u128)
            .checked_mul(token_sell_proportion)?)
        .checked_div(scaling_factor)?;
        msg!("get_sell_price: sol_received: {}", sol_received);
        let recv = <u128 as std::convert::TryInto<u64>>::try_into(sol_received)
            .ok()?
            .min(self.real_sol_reserves);

        msg!("get_sell_price: recv: {}", recv);
        Some(recv)
    }

    pub fn apply_sell(&mut self, token_amount: u64) -> Option<SellResult> {
        msg!("apply_sell: token_amount: {}", token_amount);
        let new_virtual_token_reserves =
            (self.virtual_token_reserves as u128).checked_add(token_amount as u128)?;
        msg!(
            "apply_sell: new_virtual_token_reserves: {}",
            new_virtual_token_reserves
        );
        let new_real_token_reserves =
            (self.real_token_reserves as u128).checked_add(token_amount as u128)?;
        msg!(
            "apply_sell: new_real_token_reserves: {}",
            new_real_token_reserves
        );

        let sol_amount = self.get_sell_price(token_amount)?;
        msg!("apply_sell: sol_amount: {}", sol_amount);

        let new_virtual_sol_reserves =
            (self.virtual_sol_reserves as u128).checked_sub(sol_amount as u128)?;
        msg!(
            "apply_sell: new_virtual_sol_reserves: {}",
            new_virtual_sol_reserves
        );
        let new_real_sol_reserves =
            (self.real_sol_reserves as u128).checked_sub(sol_amount as u128)?;
        msg!(
            "apply_sell: new_real_sol_reserves: {}",
            new_real_sol_reserves
        );

        self.virtual_token_reserves = new_virtual_token_reserves.try_into().ok()?;
        msg!(
            "apply_sell: updated virtual_token_reserves: {}",
            self.virtual_token_reserves
        );
        self.real_token_reserves = new_real_token_reserves.try_into().ok()?;
        msg!(
            "apply_sell: updated real_token_reserves: {}",
            self.real_token_reserves
        );
        self.virtual_sol_reserves = new_virtual_sol_reserves.try_into().ok()?;
        msg!(
            "apply_sell: updated virtual_sol_reserves: {}",
            self.virtual_sol_reserves
        );
        self.real_sol_reserves = new_real_sol_reserves.try_into().ok()?;
        msg!(
            "apply_sell: updated real_sol_reserves: {}",
            self.real_sol_reserves
        );
        self.msg();
        Some(SellResult {
            token_amount,
            sol_amount,
        })
    }

    pub fn get_tokens_for_buy_sol(&self, sol_amount: u64) -> Option<u64> {
        msg!("GetTokensForBuySol: sol_amount: {}", sol_amount);
        if sol_amount == 0 {
            return None;
        }
        msg!("GetTokensForBuySol: sol_amount: {}", sol_amount);

        let product_of_reserves =
            (self.virtual_sol_reserves as u128).checked_mul(self.virtual_token_reserves as u128)?;
        msg!(
            "GetTokensForBuySol: product_of_reserves: {}",
            product_of_reserves
        );
        let new_virtual_sol_reserves =
            (self.virtual_sol_reserves as u128).checked_add(sol_amount as u128)?;
        msg!(
            "GetTokensForBuySol: new_virtual_sol_reserves: {}",
            new_virtual_sol_reserves
        );
        let new_virtual_token_reserves = product_of_reserves
            .checked_div(new_virtual_sol_reserves)?
            .checked_add(1)?;
        msg!(
            "GetTokensForBuySol: new_virtual_token_reserves: {}",
            new_virtual_token_reserves
        );
        let tokens_received =
            (self.virtual_token_reserves as u128).checked_sub(new_virtual_token_reserves)?;
        msg!("GetTokensForBuySol: tokens_received: {}", tokens_received);
        Some(
            <u128 as std::convert::TryInto<u64>>::try_into(tokens_received)
                .ok()?
                .min(self.real_token_reserves),
        )
    }

    pub fn get_tokens_for_sell_sol(&self, sol_amount: u64) -> Option<u64> {
        msg!("GetTokensForSellSol: sol_amount: {}", sol_amount);
        if sol_amount == 0 || sol_amount > self.real_sol_reserves {
            msg!("GetTokensForSellSol: sol_amount is invalid");
            return None;
        }

        let scaling_factor = self.initial_virtual_token_reserves as u128;

        let scaled_sol = (sol_amount as u128).checked_mul(scaling_factor)?;
        msg!("GetTokensForSellSol: scaled_sol: {}", scaled_sol);
        let sol_sell_proportion = scaled_sol.checked_div(self.virtual_sol_reserves as u128)?;
        msg!(
            "GetTokensForSellSol: sol_sell_proportion: {}",
            sol_sell_proportion
        );
        let tokens_received = ((self.virtual_token_reserves as u128)
            .checked_mul(sol_sell_proportion)?)
        .checked_div(scaling_factor)?;
        msg!("GetTokensForSellSol: tokens_received: {}", tokens_received);

        tokens_received.try_into().ok()
    }

    pub fn is_started(&self, clock: &Clock) -> bool {
        let now = clock.unix_timestamp;
        now >= self.start_time
    }

    pub fn msg(&self) -> () {
        msg!("{:#?}", self);
    }

    pub fn invariant<'a>(&self, lamports: &u64, tkn_balance: &u64) -> Result<()> {
        let rent_exemption_balance: u64 =
            Rent::get()?.minimum_balance(8 + BondingCurve::INIT_SPACE as usize);
        let bonding_curve_pool_lamports: u64 = lamports - rent_exemption_balance;

        // Ensure real sol reserves are equal to bonding curve pool lamports
        if bonding_curve_pool_lamports != self.real_sol_reserves {
            msg!("Invariant failed: real_sol_reserves != bonding_curve_pool_lamports");
            return Err(ProgramError::BondingCurveInvariant.into());
        }

        // Ensure the virtual reserves are always positive
        if self.virtual_sol_reserves <= 0 {
            msg!("Invariant failed: virtual_sol_reserves <= 0");
            return Err(ProgramError::BondingCurveInvariant.into());
        }
        if self.virtual_token_reserves <= 0 {
            msg!("Invariant failed: virtual_token_reserves <= 0");
            return Err(ProgramError::BondingCurveInvariant.into());
        }

        // Ensure the token total supply is consistent with the reserves
        if self.real_token_reserves != *tkn_balance {
            msg!("Invariant failed: real_token_reserves != tkn_balance");
            msg!("real_token_reserves: {}", self.real_token_reserves);
            msg!("tkn_balance: {}", tkn_balance);
            return Err(ProgramError::BondingCurveInvariant.into());
        }

        // Ensure the bonding curve is complete only if real token reserves are zero
        if self.complete && self.real_token_reserves != 0 {
            msg!("Invariant failed: bonding curve marked as complete but real_token_reserves != 0");
            return Err(ProgramError::BondingCurveInvariant.into());
        }

        Ok(())
    }
}
impl fmt::Display for BondingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BondingCurve {{ creator: {:?}, initial_virtual_token_reserves: {:?}, virtual_sol_reserves: {:?}, virtual_token_reserves: {:?}, real_sol_reserves: {:?}, real_token_reserves: {:?}, token_total_supply: {:?}, presale_supply: {:?}, bonding_supply: {:?}, sol_launch_threshold: {:?}, start_time: {:?}, complete: {:?}, allocation: \n{:?} \n}}",
            self.creator,
            self.initial_virtual_token_reserves,
            self.virtual_sol_reserves, self.virtual_token_reserves, self.real_sol_reserves,
            self.real_token_reserves, self.token_total_supply, self.presale_supply,
            self.bonding_supply, self.sol_launch_threshold, self.start_time, self.complete,
            self.allocation
        )
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::{Clock, Pubkey};
    use once_cell::sync::Lazy;

    use crate::state::{
        allocation::AllocationData,
        bonding_curve::{BondingCurve, CreateBondingCurveParams},
    };
    use std::{
        ops::Mul,
        time::{SystemTime, UNIX_EPOCH},
    };
    static START_TIME: Lazy<i64> = Lazy::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });
    static SOL_LAUNCH_THRESHOLD: Lazy<u64> = Lazy::new(|| 70u64.mul(10u64.pow(9)));
    static CLOCK: Lazy<Clock> = Lazy::new(|| Clock {
        unix_timestamp: START_TIME.clone(),
        ..Clock::default()
    });
    #[test]
    fn test_buy_and_sell_too_much() {
        let creator = Pubkey::default();
        let allocation = AllocationData::default();

        let params = CreateBondingCurveParams {
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 2000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            virtual_token_multiplier: 7.3,
            virtual_sol_reserves: 600,

            allocation,
        };
        let mut bc = BondingCurve::default();
        let mut curve = bc.update_from_params(creator, &params, &CLOCK);
        let curve_initial = curve.clone();
        // Attempt to buy more tokens than available in reserves
        let buy_result = curve.apply_buy(2000).unwrap();
        println!("{:?} \n", buy_result);
        assert_eq!(buy_result.token_amount, 825); // Adjusted based on available tokens
        assert_eq!(buy_result.sol_amount, 2000);
        assert_eq!(
            curve.real_token_reserves,
            curve_initial.real_token_reserves - buy_result.token_amount
        );
        assert_eq!(
            curve.virtual_token_reserves,
            curve_initial.virtual_token_reserves - buy_result.token_amount as u128
        );
        assert_eq!(
            curve.real_sol_reserves,
            curve_initial.real_sol_reserves + buy_result.sol_amount
        );
        assert_eq!(
            curve.virtual_sol_reserves,
            curve_initial.virtual_sol_reserves + buy_result.sol_amount
        );
        println!("{} \n", curve);
        println!("{:?} \n", buy_result);

        // Attempt to sell more tokens than available in reserves
        let sell_result = curve.apply_sell(2000);
        assert!(sell_result.is_none());
        println!("{} \n", curve);
        println!("{:?} \n", sell_result);
    }

    #[test]
    fn test_apply_sell() {
        let creator = Pubkey::default();
        let allocation = AllocationData::default();

        let params = CreateBondingCurveParams {
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 2000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            virtual_token_multiplier: 7.3,
            virtual_sol_reserves: 600,

            allocation,
        };
        let mut bc = BondingCurve::default();
        let mut curve = bc.update_from_params(creator, &params, &CLOCK);
        // first apply buy
        curve.apply_buy(1000).unwrap();

        // let curve_initial = curve.clone();
        let result = curve.apply_sell(200).unwrap();
        println!("{:?} \n", result);
        assert_eq!(result.token_amount, 200);
        assert_eq!(result.sol_amount, 793);
        assert_eq!(curve.virtual_token_reserves, 603);
        assert_eq!(curve.real_token_reserves, 530);
        assert_eq!(curve.virtual_sol_reserves, 807);
        assert_eq!(curve.real_sol_reserves, 207);
    }

    #[test]
    fn test_get_sell_price() {
        let creator = Pubkey::default();
        let allocation = AllocationData::default();

        let params = CreateBondingCurveParams {
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 2000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            virtual_token_multiplier: 7.3,
            virtual_sol_reserves: 600,

            allocation,
        };
        let mut bc = BondingCurve::default();
        let mut curve = bc.update_from_params(creator, &params, &CLOCK);
        // first apply buy
        curve.apply_buy(1000).unwrap();

        // let curve_initial = curve.clone();
        // Edge case: zero tokens
        assert_eq!(curve.get_sell_price(0), None);

        // Normal case
        assert_eq!(curve.get_sell_price(396), Some(1000));

        // Should not exceed real sol reserves
        assert_eq!(curve.get_sell_price(5000), None);
    }

    #[test]
    fn test_apply_buy() {
        let creator = Pubkey::default();
        let allocation = AllocationData::default();

        let params = CreateBondingCurveParams {
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 2000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            virtual_token_multiplier: 7.3,
            virtual_sol_reserves: 600,

            allocation,
        };
        let mut bc = BondingCurve::default();
        let mut curve = bc.update_from_params(creator, &params, &CLOCK);
        let curve_initial = curve.clone();

        let purchase_amount = 100;

        let result = curve.apply_buy(purchase_amount).unwrap();
        println!("{:?} \n", result);
        assert_eq!(result.sol_amount, purchase_amount);
        assert_eq!(result.token_amount, 153);
        assert_eq!(
            curve.virtual_token_reserves,
            curve_initial.virtual_token_reserves - result.token_amount as u128
        );
        assert_eq!(
            curve.real_token_reserves,
            curve_initial.real_token_reserves - result.token_amount
        );
        assert_eq!(curve.virtual_sol_reserves, 700); // Adjusted based on purchased SOL
        assert_eq!(curve.real_sol_reserves, purchase_amount); // Adjusted based on purchased SOL
    }

    #[test]
    fn test_get_buy_price() {
        let creator = Pubkey::default();
        let allocation = AllocationData::default();

        let params = CreateBondingCurveParams {
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 2000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            virtual_token_multiplier: 7.3,
            virtual_sol_reserves: 600,

            allocation,
        };
        let mut bc = BondingCurve::default();
        let curve = bc.update_from_params(creator, &params, &CLOCK);
        // let _curve_initial = curve.clone();
        assert_eq!(curve.get_buy_price(0), None);

        // Normal case
        assert_eq!(curve.get_buy_price(100), Some(62));

        // Edge case: very large token amount
        assert_eq!(curve.get_buy_price(2000), None);
    }

    #[test]
    fn test_get_tokens_for_buy_sol() {
        let creator = Pubkey::default();
        let allocation = AllocationData::default();

        let params = CreateBondingCurveParams {
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 2000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            virtual_token_multiplier: 7.3,
            virtual_sol_reserves: 600,

            allocation,
        };
        let mut bc = BondingCurve::default();
        let curve = bc.update_from_params(creator, &params, &CLOCK);
        // let _curve_initial = curve.clone();

        // Test case 1: Normal case
        assert_eq!(curve.get_tokens_for_buy_sol(100), Some(153)); // Adjusted based on current method logic

        // Test case 2: Edge case - zero SOL
        assert_eq!(curve.get_tokens_for_buy_sol(0), None);

        // Test case 4: Large SOL amount (but within limits)
        assert_eq!(curve.get_tokens_for_buy_sol(3000), Some(894));

        // Test case 5: SOL amount that would exceed real token reserves
        assert_eq!(
            curve.get_tokens_for_buy_sol(900000),
            Some(curve.bonding_supply)
        );
    }

    #[test]
    fn test_get_tokens_for_sell_sol() {
        let creator = Pubkey::default();
        let allocation = AllocationData::default();

        let params = CreateBondingCurveParams {
            name: "test".to_string(),
            symbol: "test".to_string(),
            uri: "test".to_string(),
            start_time: Some(*START_TIME),

            token_total_supply: 2000,
            sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

            virtual_token_multiplier: 7.3,
            virtual_sol_reserves: 600,

            allocation,
        };
        let mut bc = BondingCurve::default();
        let mut curve = bc.update_from_params(creator, &params, &CLOCK);
        // let _curve_initial = curve.clone();
        // first apply buy
        curve.apply_buy(1000).unwrap();

        // Test case 1: Normal case
        assert_eq!(curve.get_tokens_for_sell_sol(100), Some(25));

        // Test case 2: Edge case - zero SOL
        assert_eq!(curve.get_tokens_for_sell_sol(0), None);

        // Test case 3: Edge case - more SOL than virtual reserves
        assert_eq!(curve.get_tokens_for_sell_sol(1001), None);

        // Test case 4: Large SOL amount (but within limits)
        assert_eq!(curve.get_tokens_for_sell_sol(500), Some(125));
    }

    // FUZZ TESTS
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn fuzz_test_default_alloc_simple_curve_apply_buy(
            virtual_sol_reserves in 1..u64::MAX,
            token_total_supply in 1..u64::MAX,
            sol_amount in 1..u64::MAX,
            virtual_token_multiplier in 0.0..100.0,
            // virtual_token_reserves in 1..u64::MAX,
            // real_sol_reserves in 1..u64::MAX,
            // initial_virtual_token_reserves in 1..u64::MAX,
        ) {
            let creator = Pubkey::default();
            let allocation = AllocationData::default();

            let params = CreateBondingCurveParams {
                name: "test".to_string(),
                symbol: "test".to_string(),
                uri: "test".to_string(),
                start_time: Some(*START_TIME),

                token_total_supply,
                sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

                virtual_token_multiplier,
                virtual_sol_reserves,

                allocation,
            };
            let mut bc = BondingCurve::default();
            let mut curve = bc.update_from_params(creator, &params, &CLOCK);
            let _curve_initial = curve.clone();

            if let Some(result) = curve.apply_buy(sol_amount) {
                prop_assert!(result.token_amount <= _curve_initial.real_token_reserves, "Token amount bought should not exceed real token reserves");
            }
        }

        #[test]
        fn fuzz_test_default_alloc_simple_curve_apply_sell(
            virtual_sol_reserves in 1..u64::MAX,
            token_total_supply in 1..u64::MAX,

            token_amount in 1..u64::MAX,
            buy_sol_amount in 1..u64::MAX,
            virtual_token_multiplier in 0.1..100.0,
            // virtual_token_reserves in 1..u64::MAX,
            // real_sol_reserves in 1..u64::MAX,
            // initial_virtual_token_reserves in 1..u64::MAX,
        ) {
            let creator = Pubkey::default();
            let allocation = AllocationData::default();

            let params = CreateBondingCurveParams {
                name: "test".to_string(),
                symbol: "test".to_string(),
                uri: "test".to_string(),
                start_time: Some(*START_TIME),

                token_total_supply,
                sol_launch_threshold: *SOL_LAUNCH_THRESHOLD,

                virtual_token_multiplier,
                virtual_sol_reserves,

                allocation,
            };
            let mut bc = BondingCurve::default();
            let curve = bc.update_from_params(creator, &params, &CLOCK);
            let buy_result = curve.apply_buy(buy_sol_amount);
            if buy_result.is_none() {
                return Ok(())
            }
            let _curve_after_buy = curve.clone();
            if let Some(result) = curve.apply_sell(token_amount) {
                prop_assert!(result.sol_amount <= _curve_after_buy.real_sol_reserves, "SOL amount to send to seller should not exceed real SOL reserves");
            }
        }

    }
}
