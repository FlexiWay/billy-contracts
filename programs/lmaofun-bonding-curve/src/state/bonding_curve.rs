use crate::errors::ProgramError;
use crate::Global;
use anchor_lang::prelude::*;
use anchor_spl::mint;
use std::fmt;
use std::ops::Mul;

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

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
    pub creator: Pubkey,
    pub initial_virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub start_time: i64,
    pub complete: bool,
}

impl BondingCurve {
    pub const SEED_PREFIX: &'static str = "bonding-curve";

    // pub fn get_signer<'a>(&'a self, bump: &'a u8, mint: &'a Pubkey) -> [&'a [u8]; 2] {
    //     let prefix_bytes = [..BondingCurve::SEED_PREFIX.as_bytes(), ..&mint.to_bytes()];
    //     let bump_slice: &'a [u8] = std::slice::from_ref(bump);
    //     // [prefix_bytes, bump_slice];
    //     [&[BondingCurve::SEED_PREFIX.as_bytes(), mint, bump_slice]]
    // }

    pub fn new_from_global(
        &mut self,
        global: &Global,
        creator: Pubkey,
        pool_start_time: i64,
    ) -> &mut Self {
        self.virtual_sol_reserves = global.initial_virtual_sol_reserves;
        self.initial_virtual_token_reserves = global.initial_virtual_token_reserves;
        self.virtual_token_reserves = global.initial_virtual_token_reserves;
        self.real_sol_reserves = global.initial_real_sol_reserves;
        self.real_token_reserves = global.initial_real_token_reserves;
        self.token_total_supply = global.initial_token_supply;
        self.creator = creator;
        self.complete = false;
        self.start_time = pool_start_time;
        self
    }

    pub fn get_buy_price(&self, tokens: u64) -> Option<u64> {
        msg!("get_buy_price: tokens: {}", tokens);
        if tokens == 0 || tokens > self.virtual_token_reserves {
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

        Some(BuyResult {
            token_amount: final_token_amount,
            sol_amount,
        })
    }

    pub fn get_sell_price(&self, tokens: u64) -> Option<u64> {
        msg!("get_sell_price: tokens: {}", tokens);
        if tokens == 0 || tokens > self.virtual_token_reserves {
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

        Some(
            <u128 as std::convert::TryInto<u64>>::try_into(sol_received)
                .ok()?
                .min(self.real_sol_reserves),
        )
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
        msg!("creator: {}", self.creator);
        msg!(
            "initial_virtual_token_reserves: {}",
            self.initial_virtual_token_reserves
        );
        msg!("virtual_sol_reserves: {}", self.virtual_sol_reserves);
        msg!("virtual_token_reserves: {}", self.virtual_token_reserves);
        msg!("real_sol_reserves: {}", self.real_sol_reserves);
        msg!("real_token_reserves: {}", self.real_token_reserves);
        msg!("token_total_supply: {}", self.token_total_supply);
        msg!("complete: {}", self.complete);
        msg!("start_time: {}", self.start_time);
    }

    pub fn invariant(bonding_curve_acc: &Account<BondingCurve>) -> Result<()> {
        let rent_exemption_balance: u64 =
            Rent::get()?.minimum_balance(8 + BondingCurve::INIT_SPACE as usize);
        let bonding_curve_total_lamports: u64 = bonding_curve_acc.get_lamports();
        let bonding_curve_pool_lamports: u64 =
            bonding_curve_total_lamports - rent_exemption_balance;

        // Ensure real sol reserves are equal to bonding curve pool lamports
        if bonding_curve_pool_lamports != bonding_curve_acc.real_sol_reserves {
            msg!("Invariant failed: real_sol_reserves != bonding_curve_pool_lamports");
            return Err(ProgramError::BondingCurveInvariant.into());
        }

        // Ensure the virtual reserves are always positive
        if bonding_curve_acc.virtual_sol_reserves <= 0 {
            msg!("Invariant failed: virtual_sol_reserves <= 0");
            return Err(ProgramError::BondingCurveInvariant.into());
        }
        if bonding_curve_acc.virtual_token_reserves <= 0 {
            msg!("Invariant failed: virtual_token_reserves <= 0");
            return Err(ProgramError::BondingCurveInvariant.into());
        }

        // Ensure the token total supply is consistent with the reserves
        if bonding_curve_acc.token_total_supply < bonding_curve_acc.real_token_reserves {
            msg!("Invariant failed: token_total_supply < real_token_reserves");
            return Err(ProgramError::BondingCurveInvariant.into());
        }

        // Ensure the bonding curve is complete only if real token reserves are zero
        if bonding_curve_acc.complete && bonding_curve_acc.real_token_reserves != 0 {
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
            "virtual_sol_reserves: {}, virtual_token_reserves: {}, real_sol_reserves: {}, real_token_reserves: {}, token_total_supply: {}, complete: {}",
            self.virtual_sol_reserves,
            self.virtual_token_reserves,
            self.real_sol_reserves,
            self.real_token_reserves,
            self.token_total_supply,
            self.complete
        )
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use once_cell::sync::Lazy;

    use crate::state::bonding_curve::BondingCurve;
    use std::time::{SystemTime, UNIX_EPOCH};
    static START_TIME: Lazy<i64> = Lazy::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });
    #[test]
    fn test_buy_and_sell_too_much() {
        let virtual_sol_reserves = 600;
        let virtual_token_reserves = 600;
        let real_sol_reserves = 0;
        let real_token_reserves: u64 = 500;
        let initial_virtual_token_reserves = 1000;
        let token_total_supply = 1000;
        let creator = Pubkey::default();
        let complete = false;
        let mut curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
            start_time: *START_TIME,
        };

        // Attempt to buy more tokens than available in reserves
        let buy_result = curve.apply_buy(2000).unwrap();
        println!("{:?} \n", buy_result);
        assert_eq!(buy_result.token_amount, 461); // Adjusted based on available tokens
        assert_eq!(buy_result.sol_amount, 2000);
        assert_eq!(
            curve.real_token_reserves,
            real_token_reserves - buy_result.token_amount
        );
        assert_eq!(
            curve.virtual_token_reserves,
            virtual_token_reserves - buy_result.token_amount
        );
        assert_eq!(
            curve.real_sol_reserves,
            real_sol_reserves + buy_result.sol_amount
        );
        assert_eq!(
            curve.virtual_sol_reserves,
            virtual_sol_reserves + buy_result.sol_amount
        );
        println!("{} \n", curve);
        println!("{:?} \n", buy_result);

        // Attempt to sell more tokens than available in reserves
        let sell_result = curve.apply_sell(2000);
        assert!(sell_result.is_none());
        // assert_eq!(sell_result.token_amount, 2000); // Should sell requested amount
        // assert_eq!(sell_result.sol_amount, 2000); // Adjusted expected result
        // assert_eq!(curve.real_sol_reserves, 0);
        // assert_eq!(curve.virtual_sol_reserves, 600);
        // assert_eq!(curve.real_token_reserves, 2090); // Adjusted based on sold tokens
        // assert_eq!(curve.virtual_token_reserves, 2100);
        println!("{} \n", curve);
        println!("{:?} \n", sell_result);
    }

    #[test]
    fn test_apply_sell() {
        let virtual_sol_reserves = 1000;
        let virtual_token_reserves = 1000;
        let real_sol_reserves = 500;
        let real_token_reserves: u64 = 500;
        let initial_virtual_token_reserves = 1000;
        let token_total_supply = 1000;
        let creator = Pubkey::default();
        let complete = false;

        let mut curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
            start_time: *START_TIME,
        };
        let result = curve.apply_sell(100).unwrap();
        println!("{:?} \n", result);
        assert_eq!(result.token_amount, 100);
        assert_eq!(result.sol_amount, 100);
        assert_eq!(curve.virtual_token_reserves, 1100);
        assert_eq!(curve.real_token_reserves, 600);
        assert_eq!(curve.virtual_sol_reserves, 900);
        assert_eq!(curve.real_sol_reserves, 400);
    }

    #[test]
    fn test_get_sell_price() {
        let virtual_sol_reserves = 1000;
        let virtual_token_reserves = 1000;
        let real_sol_reserves = 500;
        let real_token_reserves: u64 = 500;
        let initial_virtual_token_reserves = 1000;
        let token_total_supply = 1000;
        let creator = Pubkey::default();
        let complete = false;

        let curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
            start_time: *START_TIME,
        };

        // Edge case: zero tokens
        assert_eq!(curve.get_sell_price(0), None);

        // Normal case
        assert_eq!(curve.get_sell_price(100), Some(100));

        // Should not exceed real sol reserves
        assert_eq!(curve.get_sell_price(5000), None);
    }

    #[test]
    fn test_apply_buy() {
        let virtual_sol_reserves = 600;
        let virtual_token_reserves = 600;
        let real_sol_reserves = 500;
        let real_token_reserves = 500;
        let initial_virtual_token_reserves = 1000;
        let token_total_supply = 1000;
        let creator = Pubkey::default();
        let complete = false;

        let mut curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
            start_time: *START_TIME,
        };

        let purchase_amount = 100;

        let result = curve.apply_buy(100).unwrap();
        println!("{:?} \n", result);
        assert_eq!(result.sol_amount, purchase_amount);
        assert_eq!(result.token_amount, 85);
        assert_eq!(
            curve.virtual_token_reserves,
            virtual_token_reserves - result.token_amount
        );
        assert_eq!(
            curve.real_token_reserves,
            real_token_reserves - result.token_amount
        );
        assert_eq!(curve.virtual_sol_reserves, 700); // Adjusted based on purchased SOL
        assert_eq!(curve.real_sol_reserves, 600); // Adjusted based on purchased SOL
    }

    #[test]
    fn test_get_buy_price() {
        let virtual_sol_reserves = 1000;
        let virtual_token_reserves = 1000;
        let real_sol_reserves = 500;
        let real_token_reserves = 500;
        let initial_virtual_token_reserves = 1000;
        let token_total_supply = 1000;
        let creator = Pubkey::default();
        let complete = false;

        let curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
            start_time: *START_TIME,
        };

        assert_eq!(curve.get_buy_price(0), None);

        // Normal case
        assert_eq!(curve.get_buy_price(100), Some(112));

        // Edge case: very large token amount
        assert_eq!(curve.get_buy_price(2000), None);
    }

    #[test]
    fn test_get_tokens_for_buy_sol() {
        let virtual_sol_reserves = 1000;
        let virtual_token_reserves = 1000;
        let real_sol_reserves = 500;
        let real_token_reserves = 500;
        let initial_virtual_token_reserves = 1000;
        let token_total_supply = 1000;
        let creator = Pubkey::default();
        let complete = false;

        let curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
            start_time: *START_TIME,
        };

        // Test case 1: Normal case
        assert_eq!(curve.get_tokens_for_buy_sol(100), Some(90)); // Adjusted based on current method logic

        // Test case 2: Edge case - zero SOL
        assert_eq!(curve.get_tokens_for_buy_sol(0), None);

        // Test case 3: Edge case - more SOL than virtual reserves
        assert_eq!(curve.get_tokens_for_buy_sol(1001), Some(500));

        // Test case 4: Large SOL amount (but within limits)
        assert_eq!(curve.get_tokens_for_buy_sol(500), Some(333));

        // Test case 5: SOL amount that would exceed real token reserves
        assert_eq!(curve.get_tokens_for_buy_sol(900), Some(473));
    }

    #[test]
    fn test_get_tokens_for_sell_sol() {
        let virtual_sol_reserves = 1000;
        let virtual_token_reserves = 1000;
        let real_sol_reserves = 500;
        let real_token_reserves = 500;
        let initial_virtual_token_reserves = 1000;
        let token_total_supply = 1000;
        let creator = Pubkey::default();
        let complete = false;

        let curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
            start_time: *START_TIME,
        };
        // Test case 1: Normal case
        assert_eq!(curve.get_tokens_for_sell_sol(100), Some(100)); // Adjusted based on current method logic

        // Test case 2: Edge case - zero SOL
        assert_eq!(curve.get_tokens_for_sell_sol(0), None);

        // Test case 3: Edge case - more SOL than virtual reserves
        assert_eq!(curve.get_tokens_for_sell_sol(1001), None);

        // Test case 4: Large SOL amount (but within limits)
        assert_eq!(curve.get_tokens_for_sell_sol(500), Some(500));

        // Test case 5: SOL amount that would exceed real token reserves
        assert_eq!(curve.get_tokens_for_sell_sol(900), None);
    }

    // fuzz

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn fuzz_test_apply_buy(
            virtual_sol_reserves in 1..u64::MAX,
            virtual_token_reserves in 1..u64::MAX,
            real_sol_reserves in 1..u64::MAX,
            real_token_reserves in 1..u64::MAX,
            initial_virtual_token_reserves in 1..u64::MAX,
            token_total_supply in 1..u64::MAX,
            sol_amount in 1..u64::MAX,
        ) {
            let creator = Pubkey::default();
            let complete = false;

            let mut curve = BondingCurve {
                virtual_sol_reserves,
                virtual_token_reserves,
                real_sol_reserves,
                real_token_reserves,
                initial_virtual_token_reserves,
                token_total_supply,
                creator,
                complete,
                start_time: *START_TIME,
            };

            if let Some(result) = curve.apply_buy(sol_amount) {
                prop_assert!(result.token_amount <= real_token_reserves, "Token amount bought should not exceed real token reserves");
            }
        }

        #[test]
        fn fuzz_test_apply_sell(
            virtual_sol_reserves in 1..u64::MAX,
            virtual_token_reserves in 1..u64::MAX,
            real_sol_reserves in 1..u64::MAX,
            real_token_reserves in 1..u64::MAX,
            initial_virtual_token_reserves in 1..u64::MAX,
            token_total_supply in 1..u64::MAX,
            token_amount in 1..u64::MAX,
        ) {
            let creator = Pubkey::default();
            let complete = false;

            let mut curve = BondingCurve {
                virtual_sol_reserves,
                virtual_token_reserves,
                real_sol_reserves,
                real_token_reserves,
                initial_virtual_token_reserves,
                token_total_supply,
                creator,
                complete,
                start_time: *START_TIME,
            };

            if let Some(result) = curve.apply_sell(token_amount) {
                prop_assert!(result.sol_amount <= real_sol_reserves, "SOL amount to send to seller should not exceed real SOL reserves");
            }
        }
    }
}
