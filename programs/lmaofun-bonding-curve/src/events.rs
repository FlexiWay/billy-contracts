use anchor_lang::prelude::*;

#[event]
pub struct SetParamsEvent {
    pub fee_recipient: Pubkey,
    pub withdraw_authority: Pubkey,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub initial_token_supply: u64,
    pub fee_basis_points: u32,
}
