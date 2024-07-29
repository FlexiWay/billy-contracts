use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace, Debug, Default)]
pub struct CreatorDistributor {}
impl CreatorDistributor {
    pub const SEED_PREFIX: &'static str = "creator-distributor-data";

    pub fn get_signer<'a>(bump: &'a u8, mint: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }
}

#[account]
#[derive(InitSpace, Debug, Default)]
pub struct PlatformDistributor {}
impl PlatformDistributor {
    pub const SEED_PREFIX: &'static str = "platform-distributor-data";

    pub fn get_signer<'a>(bump: &'a u8, mint: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }
}

#[account]
#[derive(InitSpace, Debug, Default)]
pub struct BrandDistributor {}
impl BrandDistributor {
    pub const SEED_PREFIX: &'static str = "brand-distributor-data";

    pub fn get_signer<'a>(bump: &'a u8, mint: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }
}
