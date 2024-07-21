use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GlobalSettingsInput {
    pub initial_token_supply: u64,
    pub initial_real_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_virtual_token_reserves: u64,
    pub sol_launch_threshold: u64,

    pub fee_basis_points: u32,
}

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
}

impl Global {
    pub const SEED_PREFIX: &'static [u8; 6] = b"global";

    pub fn update_settings(&mut self, params: GlobalSettingsInput) {
        self.initial_token_supply = params.initial_token_supply;
        self.initial_real_sol_reserves = params.initial_real_sol_reserves;
        self.initial_real_token_reserves = params.initial_real_token_reserves;
        self.initial_virtual_sol_reserves = params.initial_virtual_sol_reserves;
        self.initial_virtual_token_reserves = params.initial_virtual_token_reserves;
        self.sol_launch_threshold = params.sol_launch_threshold;

        self.fee_basis_points = params.fee_basis_points;
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
