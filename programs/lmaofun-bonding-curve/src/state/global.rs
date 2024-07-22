use anchor_lang::prelude::*;

use crate::events::{GlobalUpdateEvent, IntoEvent};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GlobalAuthorityInput {
    pub global_authority: Option<Pubkey>,
    pub fee_recipient: Option<Pubkey>,
    pub withdraw_authority: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, Debug)]
pub enum ProgramStatus {
    Running,
    SwapOnly,
    SwapOnlyNoLaunch,
    Paused,
}

#[account]
#[derive(InitSpace, Debug)]
pub struct Global {
    pub status: ProgramStatus,
    pub initialized: bool,

    pub global_authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub withdraw_authority: Pubkey,

    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub initial_real_sol_reserves: u64,
    pub initial_token_supply: u64,
    pub sol_launch_threshold: u64,

    pub fee_basis_points: u32,
    pub created_mint_decimals: u8,
}
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GlobalSettingsInput {
    pub initial_token_supply: Option<u64>,
    pub initial_real_sol_reserves: Option<u64>,
    pub initial_real_token_reserves: Option<u64>,
    pub initial_virtual_sol_reserves: Option<u64>,
    pub initial_virtual_token_reserves: Option<u64>,
    pub sol_launch_threshold: Option<u64>,

    pub fee_basis_points: Option<u32>,
    pub created_mint_decimals: Option<u8>,
}

impl Global {
    pub const SEED_PREFIX: &'static [u8; 6] = b"global";

    pub fn update_settings(&mut self, params: GlobalSettingsInput) {
        if let Some(value) = params.initial_token_supply {
            self.initial_token_supply = value;
        }
        if let Some(value) = params.initial_real_sol_reserves {
            self.initial_real_sol_reserves = value;
        }
        if let Some(value) = params.initial_real_token_reserves {
            self.initial_real_token_reserves = value;
        }
        if let Some(value) = params.initial_virtual_sol_reserves {
            self.initial_virtual_sol_reserves = value;
        }
        if let Some(value) = params.initial_virtual_token_reserves {
            self.initial_virtual_token_reserves = value;
        }
        if let Some(value) = params.sol_launch_threshold {
            self.sol_launch_threshold = value;
        }

        if let Some(fee_basis_points) = params.fee_basis_points {
            self.fee_basis_points = fee_basis_points;
        }

        if let Some(created_mint_decimals) = params.created_mint_decimals {
            self.created_mint_decimals = created_mint_decimals;
        }
    }

    pub fn update_authority(&mut self, params: GlobalAuthorityInput) {
        if let Some(global_authority) = params.global_authority {
            self.global_authority = global_authority;
        }
        if let Some(fee_recipient) = params.fee_recipient {
            self.fee_recipient = fee_recipient;
        }
        if let Some(withdraw_authority) = params.withdraw_authority {
            self.withdraw_authority = withdraw_authority;
        }
    }
}

impl IntoEvent<GlobalUpdateEvent> for Global {
    fn into_event(&self) -> GlobalUpdateEvent {
        GlobalUpdateEvent {
            fee_recipient: self.fee_recipient,
            withdraw_authority: self.withdraw_authority,
            initial_virtual_token_reserves: self.initial_virtual_token_reserves,
            initial_virtual_sol_reserves: self.initial_virtual_sol_reserves,
            initial_real_token_reserves: self.initial_real_token_reserves,
            initial_token_supply: self.initial_token_supply,
            fee_basis_points: self.fee_basis_points,
            sol_launch_threshold: self.sol_launch_threshold,
            created_mint_decimals: self.created_mint_decimals,
        }
    }
}
