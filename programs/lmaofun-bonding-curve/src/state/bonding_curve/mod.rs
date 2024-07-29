pub mod implementation;
pub mod tests;
use crate::state::allocation::AllocationData;
use anchor_lang::prelude::*;

use super::allocation::AllocationDataParams;

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

#[account]
#[derive(InitSpace, Debug, Default)]
pub struct BondingCurve {
    pub creator: Pubkey,
    pub platform_authority: Pubkey,
    pub brand_authority: Pubkey,

    pub virtual_token_multiplier_bps: u64,

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

    pub virtual_token_multiplier_bps: u64,
    pub virtual_sol_reserves: u64, // should this be fixed instead?

    pub allocation: AllocationDataParams,
}
