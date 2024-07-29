use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace, Debug, Default)]
pub struct Creator {}
impl Creator {
    pub const SEED_PREFIX: &'static str = "creator-data";

    pub fn get_signer<'a>(bump: &'a u8, mint: &'a Pubkey) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }
}
