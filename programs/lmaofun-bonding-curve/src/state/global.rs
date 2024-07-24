use crate::events::{GlobalUpdateEvent, IntoEvent};
use anchor_lang::prelude::*;
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GlobalAuthorityInput {
    pub global_authority: Option<Pubkey>,
    pub withdraw_authority: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, Debug, PartialEq)]
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
    pub withdraw_authority: Pubkey,

    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub initial_real_sol_reserves: u64,
    pub initial_token_supply: u64,
    pub sol_launch_threshold: u64,

    pub trade_fee_bps: u32,
    pub launch_fee_lamports: u64,

    pub created_mint_decimals: u8,
}
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct GlobalSettingsInput {
    pub initial_token_supply: Option<u64>,
    pub initial_real_sol_reserves: Option<u64>,
    pub initial_real_token_reserves: Option<u64>,
    pub initial_virtual_sol_reserves: Option<u64>,
    pub initial_virtual_token_reserves: Option<u64>,
    pub sol_launch_threshold: Option<u64>,

    pub trade_fee_bps: Option<u32>,
    pub created_mint_decimals: Option<u8>,
    pub launch_fee_lamports: Option<u64>,

    pub status: Option<ProgramStatus>,
}

impl Global {
    pub const SEED_PREFIX: &str = "global";

    pub fn get_signer<'a>(&'a self, bump: &'a u8) -> [&'a [u8]; 2] {
        let prefix_bytes = Global::SEED_PREFIX.as_bytes();
        let bump_slice: &'a [u8] = std::slice::from_ref(bump);
        [prefix_bytes, bump_slice]
    }

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
        if let Some(trade_fee_bps) = params.trade_fee_bps {
            self.trade_fee_bps = trade_fee_bps;
        }
        if let Some(launch_fee_lamports) = params.launch_fee_lamports {
            self.launch_fee_lamports = launch_fee_lamports;
        }
        if let Some(created_mint_decimals) = params.created_mint_decimals {
            self.created_mint_decimals = created_mint_decimals;
        }
        if let Some(status) = params.status {
            self.status = status;
        }
    }

    pub fn update_authority(&mut self, params: GlobalAuthorityInput) {
        if let Some(global_authority) = params.global_authority {
            self.global_authority = global_authority;
        }
        if let Some(withdraw_authority) = params.withdraw_authority {
            self.withdraw_authority = withdraw_authority;
        }
    }
}

impl IntoEvent<GlobalUpdateEvent> for Global {
    fn into_event(&self) -> GlobalUpdateEvent {
        GlobalUpdateEvent {
            global_authority: self.global_authority,
            initial_virtual_token_reserves: self.initial_virtual_token_reserves,
            initial_virtual_sol_reserves: self.initial_virtual_sol_reserves,
            initial_real_token_reserves: self.initial_real_token_reserves,
            initial_token_supply: self.initial_token_supply,
            trade_fee_bps: self.trade_fee_bps,
            sol_launch_threshold: self.sol_launch_threshold,
            created_mint_decimals: self.created_mint_decimals,
            launch_fee_lamports: self.launch_fee_lamports,
        }
    }
}
