use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
use instructions::{initialize::*, set_params::*};
use state::global::*;

#[error_code]
pub enum ContractError {
    #[msg("Invalid instruction data")]
    InvalidInstructionData,
}
declare_id!("E52KjA58odp3taqmaCuBFdDya3s4TA1ho4tSXoW2igxb");

#[program]
pub mod bonding_curve {

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        authority_params: GlobalAuthorityInput,
        settings_params: GlobalSettingsInput,
    ) -> Result<()> {
        Initialize::handler(ctx, authority_params, settings_params)
    }
    pub fn set_params(
        ctx: Context<SetParams>,
        settings_params: GlobalSettingsInput,
        authority_params: GlobalAuthorityInput,
        status: ProgramStatus,
    ) -> Result<()> {
        SetParams::handler(ctx, authority_params, settings_params, status)
    }
}
