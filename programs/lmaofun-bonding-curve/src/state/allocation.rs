use anchor_lang::prelude::*;
use anchor_lang::{AnchorDeserialize, AnchorSerialize, InitSpace};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, Debug, PartialEq)]
pub struct AllocationData {
    // percents
    pub dev: f64,
    pub cex: f64,
    pub launch_brandkit: f64,
    pub lifetime_brandkit: f64,
    pub platform: f64,
    pub presale: f64,
    pub pool_reserve: f64,
}
impl Default for AllocationData {
    fn default() -> Self {
        let _10f64 = 10f64;
        Self {
            dev: _10f64,
            cex: _10f64,
            launch_brandkit: _10f64,
            lifetime_brandkit: _10f64,
            platform: _10f64,
            presale: 0f64,
            pool_reserve: 50f64,
        }
    }
}
impl AllocationData {
    pub fn is_valid(&self) -> bool {
        let sum_is_right = self.dev
            + self.cex
            + self.launch_brandkit
            + self.lifetime_brandkit
            + self.platform
            + self.presale
            + self.pool_reserve
            == 100f64;
        sum_is_right && self.pool_reserve > 0.0
    }
}
