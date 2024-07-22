use anchor_lang::prelude::*;

#[event]
pub struct GlobalUpdateEvent {
    pub fee_recipient: Pubkey,
    pub withdraw_authority: Pubkey,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub initial_token_supply: u64,
    pub fee_basis_points: u32,
    pub sol_launch_threshold: u64,
    pub created_mint_decimals: u8,
}

pub trait IntoEvent<T: anchor_lang::Event> {
    fn into_event(&self) -> T;
}
