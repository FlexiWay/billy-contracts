use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace, Debug, Default)]
pub struct CreatorDistributor {
    pub initial_vested_supply: u64,
    pub last_distribution: Option<i64>,
}
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
pub struct PresaleDistributor {
    pub initial_vested_supply: u64,
}
impl PresaleDistributor {
    pub const SEED_PREFIX: &'static str = "presale-distributor-data";

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
pub struct PlatformDistributor {
    pub initial_vested_supply: u64,
    pub last_distribution: Option<i64>,
}
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
pub struct BrandDistributor {
    pub launch_brandkit_supply: u64,
    pub lifetime_brandkit_supply: u64,
    pub initial_vested_supply: u64,
}
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
