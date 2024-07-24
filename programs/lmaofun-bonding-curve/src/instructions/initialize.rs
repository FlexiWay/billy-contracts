use crate::{errors::ProgramError, events::*, state::global::*};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: GlobalSettingsInput)]
pub struct Initialize<'info> {
    #[account(mut)]
    authority: Signer<'info>,

    #[account(
        init,
        space = 8 + Global::INIT_SPACE,
        seeds = [Global::SEED_PREFIX],
        constraint = global.initialized != true @ ProgramError::AlreadyInitialized,
        bump,
        payer = authority,
    )]
    global: Box<Account<'info, Global>>,

    system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn handler(ctx: Context<Initialize>, params: GlobalSettingsInput) -> Result<()> {
        let global = &mut ctx.accounts.global;
        global.update_authority(GlobalAuthorityInput {
            global_authority: Some(ctx.accounts.authority.key()),
        });
        global.update_settings(params);

        if global.initial_virtual_sol_reserves == 0
            || global.initial_virtual_token_reserves == 0
            || global.created_mint_decimals == 0
            || global.initial_real_token_reserves == 0
            || global.initial_token_supply == 0
        {
            global.status = ProgramStatus::SwapOnly;
        }

        global.status = ProgramStatus::Running;
        global.initialized = true;

        emit_cpi!(global.into_event());
        msg!("Initialized global state");
        Ok(())
    }
}
