use anchor_lang::prelude::*;
use std::fmt;

use crate::Global;

#[account]
#[derive(InitSpace)]
pub struct BondingCurve {
    pub creator: Pubkey,
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
        self.virtual_token_reserves = global.initial_virtual_token_reserves;
        self.real_sol_reserves = global.initial_real_sol_reserves;
        self.real_token_reserves = global.initial_real_token_reserves;
        self.token_total_supply = global.initial_token_supply;
        self.creator = creator;
        self.complete = false;
        self
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
