use anchor_lang::prelude::*;
use anchor_lang::{AnchorDeserialize, AnchorSerialize, InitSpace};

use crate::util::BASIS_POINTS_DIVISOR;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, Debug, PartialEq)]
pub struct AllocationDataParams {
    // BASIS POINTS
    pub creator: u64,
    pub cex: u64,
    pub launch_brandkit: u64,
    pub lifetime_brandkit: u64,
    pub platform: u64,
    pub presale: u64,
    pub curve_reserve: u64,
    pub pool_reserve: u64,
}
impl Default for AllocationDataParams {
    fn default() -> Self {
        let default = AllocationData::default();
        Self {
            creator: default.creator,
            cex: default.cex,
            launch_brandkit: default.launch_brandkit,
            lifetime_brandkit: default.lifetime_brandkit,
            platform: default.platform,
            presale: default.presale,
            curve_reserve: default.curve_reserve,
            pool_reserve: default.pool_reserve,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Debug, PartialEq)]

pub struct AllocationData {
    // BASIS POINTS
    pub creator: u64,
    pub cex: u64,
    pub launch_brandkit: u64,
    pub lifetime_brandkit: u64,
    pub platform: u64,
    pub presale: u64,
    pub curve_reserve: u64,
    pub pool_reserve: u64,
}

impl From<AllocationDataParams> for AllocationData {
    fn from(params: AllocationDataParams) -> Self {
        Self {
            creator: params.creator,
            cex: params.cex,
            launch_brandkit: params.launch_brandkit,
            lifetime_brandkit: params.lifetime_brandkit,
            platform: params.platform,
            presale: params.presale,
            curve_reserve: params.curve_reserve,
            pool_reserve: params.pool_reserve,
        }
    }
}

impl Default for AllocationData {
    fn default() -> Self {
        // TODO: discuss with team
        Self {
            creator: 500,
            cex: 1000,
            launch_brandkit: 1000,
            lifetime_brandkit: 1000,
            platform: 500,
            presale: 0u64,
            curve_reserve: 3000,
            pool_reserve: 3000,
        }
    }
}
impl AllocationData {
    pub fn is_valid(&self) -> bool {
        msg!("aloc:{:#?}", self);

        let sum_is_right = self.creator
            + self.cex
            + self.launch_brandkit
            + self.lifetime_brandkit
            + self.platform
            + self.presale
            + self.curve_reserve
            + self.pool_reserve
            == BASIS_POINTS_DIVISOR;
        sum_is_right && self.curve_reserve > 0 && self.pool_reserve == self.curve_reserve
    }
}
