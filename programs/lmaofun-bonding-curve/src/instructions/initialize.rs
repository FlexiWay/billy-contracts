use crate::{errors::CurveLaunchpadError, events::*, state::global::*};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(settings_params: GlobalSettingsInput, authority_params: GlobalAuthorityInput)]
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
    pub fn handler(
        ctx: Context<Initialize>,
        authority_params: GlobalAuthorityInput,
        settings_params: GlobalSettingsInput,
    ) -> Result<()> {
        let global = &mut ctx.accounts.global;

        require!(!global.initialized, CurveLaunchpadError::AlreadyInitialized);

        global.update_authority(authority_params);
        global.update_settings(settings_params);

        global.status = ProgramStatus::Running;
        global.initialized = true;
        emit_cpi!(global.into_event());
        msg!("Initialized global state");
        Ok(())
    }
}
