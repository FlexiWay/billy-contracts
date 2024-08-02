use anchor_lang::prelude::*;

use super::BondingCurveFeeVault;

impl BondingCurveFeeVault {
    pub const SEED_PREFIX: &'static str = "bonding-curve-fee-vault";

    pub fn get_signer<'a>(bump: &'a u8, mint: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }
}
