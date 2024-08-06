use anchor_lang::prelude::*;

use crate::state::allocation::AllocationDataParams;

#[derive(Debug, Clone, AnchorSerialize, InitSpace, AnchorDeserialize)]
pub struct VestingTerms {
    pub cliff: i64,
    pub duration: i64,
}

impl Default for VestingTerms {
    fn default() -> Self {
        VestingTerms {
            cliff: 7 * 24 * 60 * 60,     // 7 days
            duration: 31 * 24 * 60 * 60, // 31 days
        }
    }
}

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateBondingCurveParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub start_time: Option<i64>,

    pub token_total_supply: u64,
    pub sol_launch_threshold: u64,

    pub virtual_token_multiplier_bps: u64,
    pub virtual_sol_reserves: u64, // should this be fixed instead?

    pub allocation: AllocationDataParams,

    pub vesting_terms: Option<VestingTerms>,
}
