use anchor_lang::prelude::*;

use crate::{state::bonding_curve::BondingCurve, Global};

#[event]
pub struct GlobalUpdateEvent {
    pub global: Global,
}

#[event]
pub struct CreateEvent {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub bonding_curve: BondingCurve,
}
