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
        bump,
        payer = authority,
    )]
    global: Box<Account<'info, Global>>,

    system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn handler(ctx: Context<Initialize>, params: GlobalSettingsInput) -> Result<()> {
        let global = &mut ctx.accounts.global;

        require!(!global.initialized, ProgramError::AlreadyInitialized);

        global.update_authority(GlobalAuthorityInput {
            global_authority: Some(ctx.accounts.authority.key()),
            fee_recipient: Some(ctx.accounts.authority.key()),
        });
        global.update_settings(params);

        global.status = ProgramStatus::Running;
        global.initialized = true;
        emit_cpi!(global.into_event());
        msg!("Initialized global state");
        Ok(())
    }
}
