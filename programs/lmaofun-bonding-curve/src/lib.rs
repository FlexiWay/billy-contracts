use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use global::GlobalSettingsInput;
use instructions::initialize::*;
use state::*;

#[error_code]
pub enum ContractError {
    #[msg("Invalid instruction data")]
    InvalidInstructionData,
}
declare_id!("E52KjA58odp3taqmaCuBFdDya3s4TA1ho4tSXoW2igxb");

#[program]
pub mod bonding_curve {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: GlobalSettingsInput) -> Result<()> {
        Initialize::handler(ctx, params)
    }
}
