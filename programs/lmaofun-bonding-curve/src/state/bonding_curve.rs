use anchor_lang::prelude::*;
use std::fmt;

use crate::Global;

#[derive(Debug)]
pub struct BuyResult {
    pub token_amount: u64,
    pub sol_amount: u64,
}

#[derive(Debug)]
pub struct SellResult {
    pub token_amount: u64,
    pub sol_amount: u64,
}

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
    pub complete: bool,
}

impl BondingCurve {
    pub const SEED_PREFIX: &'static [u8; 13] = b"bonding-curve";

    pub fn new_from_global(&mut self, global: &Global, creator: Pubkey) -> &mut Self {
        self.virtual_sol_reserves = global.initial_virtual_sol_reserves;
        self.initial_virtual_token_reserves = global.initial_virtual_token_reserves;
        self.virtual_token_reserves = global.initial_virtual_token_reserves;
        self.real_sol_reserves = global.initial_real_sol_reserves;
        self.real_token_reserves = global.initial_real_token_reserves;
        self.token_total_supply = global.initial_token_supply;
        self.creator = creator;
        self.complete = false;
        self
    }
    pub fn get_buy_price(&self, tokens: u64) -> Option<u64> {
        if tokens == 0 || tokens > self.virtual_token_reserves {
            return None;
        }

        let product_of_reserves = self
            .virtual_sol_reserves
            .checked_mul(self.virtual_token_reserves)?;
        let new_virtual_token_reserves = self.virtual_token_reserves.checked_sub(tokens)?;
        let new_virtual_sol_reserves = product_of_reserves
            .checked_div(new_virtual_token_reserves)?
            .checked_add(1)?;
        let amount_needed = new_virtual_sol_reserves.checked_sub(self.virtual_sol_reserves)?;

        Some(amount_needed)
    }

    pub fn apply_buy(&mut self, token_amount: u64) -> Option<BuyResult> {
        let final_token_amount = if token_amount > self.real_token_reserves {
            self.real_token_reserves
        } else {
            token_amount
        };

        let sol_amount = self.get_buy_price(final_token_amount)?;

        self.virtual_token_reserves = self
            .virtual_token_reserves
            .checked_sub(final_token_amount)?;
        self.real_token_reserves = self.real_token_reserves.checked_sub(final_token_amount)?;

        self.virtual_sol_reserves = self.virtual_sol_reserves.checked_add(sol_amount)?;
        self.real_sol_reserves = self.real_sol_reserves.checked_add(sol_amount)?;

        Some(BuyResult {
            token_amount: final_token_amount,
            sol_amount: sol_amount,
        })
    }

    pub fn apply_sell(&mut self, token_amount: u64) -> Option<SellResult> {
        self.virtual_token_reserves = self.virtual_token_reserves.checked_add(token_amount)?;
        self.real_token_reserves = self.real_token_reserves.checked_add(token_amount)?;

        let sol_amount = self.get_sell_price(token_amount)?;

        self.virtual_sol_reserves = self.virtual_sol_reserves.checked_sub(sol_amount)?;
        self.real_sol_reserves = self.real_sol_reserves.checked_sub(sol_amount)?;

        Some(SellResult {
            token_amount: token_amount,
            sol_amount: sol_amount,
        })
    }

    pub fn get_sell_price(&self, tokens: u64) -> Option<u64> {
        if tokens <= 0 || tokens > self.virtual_token_reserves {
            return None;
        }

        let scaling_factor = self.initial_virtual_token_reserves;

        let scaled_tokens = tokens.checked_mul(scaling_factor)?;
        let token_sell_proportion = scaled_tokens.checked_div(self.virtual_token_reserves)?;
        let sol_received = (self
            .virtual_sol_reserves
            .checked_mul(token_sell_proportion)?)
        .checked_div(scaling_factor)?;

        Some(sol_received.min(self.real_sol_reserves))
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

    use crate::{state::bonding_curve::BondingCurve, Global};

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
        };
        //println!("{} \n", 1/0);
        // Attempt to buy more tokens than available in reserves
        let buy_result = curve.apply_buy(2000).unwrap();
        println!("{:?} \n", buy_result);
        assert_eq!(buy_result.token_amount, 500); // Should buy up to available real_token_reserves
        assert_eq!(buy_result.sol_amount, 3001);
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
        let sell_result = curve.apply_sell(2000).unwrap();
        assert_eq!(sell_result.token_amount, 2000); // Should sell requested amount
        assert_eq!(sell_result.sol_amount, 3001);
        assert_eq!(curve.real_sol_reserves, 0);
        assert_eq!(curve.virtual_sol_reserves, 600);
        assert_eq!(curve.real_token_reserves, 2000);
        assert_eq!(curve.virtual_token_reserves, 2100);
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
        };
        let mut curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
        };
        let result = curve.apply_sell(100).unwrap();

        assert_eq!(result.token_amount, 100);
        assert_eq!(result.sol_amount, 90);
        assert_eq!(curve.virtual_token_reserves, 1100);
        assert_eq!(curve.real_token_reserves, 600);
        assert_eq!(curve.virtual_sol_reserves, 910);
        assert_eq!(curve.real_sol_reserves, 410);
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
        let mut curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
        };
        let curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
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
        };

        let purchase_amount = 100;

        let result = curve.apply_buy(100).unwrap();

        assert_eq!(result.token_amount, purchase_amount);
        assert_eq!(result.sol_amount, 121);
        assert_eq!(
            curve.virtual_token_reserves,
            virtual_token_reserves - purchase_amount
        );
        assert_eq!(
            curve.real_token_reserves,
            real_token_reserves - purchase_amount
        );
        assert_eq!(curve.virtual_sol_reserves, 721);
        assert_eq!(curve.real_sol_reserves, 621);
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

        let mut curve = BondingCurve {
            virtual_sol_reserves,
            virtual_token_reserves,
            real_sol_reserves,
            real_token_reserves,
            initial_virtual_token_reserves,
            token_total_supply,
            creator,
            complete,
        };

        assert_eq!(curve.get_buy_price(0), None);

        // Normal case
        assert_eq!(curve.get_buy_price(100), Some(112));

        // Edge case: very large token amount
        assert_eq!(curve.get_buy_price(2000), None);
    }
}
