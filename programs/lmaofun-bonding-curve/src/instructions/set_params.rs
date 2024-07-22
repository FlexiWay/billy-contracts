use crate::{errors::CurveLaunchpadError, events::*, state::global::*};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction( authority_params: GlobalAuthorityInput,settings_params: GlobalSettingsInput, status: ProgramStatus)]
pub struct SetParams<'info> {
    #[account(mut,
    constraint = authority.key() == global.global_authority.key() @ CurveLaunchpadError::InvalidAuthority
    )]
    authority: Signer<'info>,

    #[account(
        init,
        space = 8 + Global::INIT_SPACE,
        seeds = [Global::SEED_PREFIX],
        constraint = global.initialized == true @ CurveLaunchpadError::NotInitialized,
        bump,
        payer = authority,
    )]
    global: Box<Account<'info, Global>>,

    system_program: Program<'info, System>,
}

impl SetParams<'_> {
    pub fn handler(
        ctx: Context<SetParams>,
        authority_params: GlobalAuthorityInput,
        settings_params: GlobalSettingsInput,
        status: ProgramStatus,
    ) -> Result<()> {
        let global = &mut ctx.accounts.global;

        global.update_authority(authority_params);
        global.update_settings(settings_params);
        global.status = status;

        emit_cpi!(GlobalUpdateEvent {
            global: global.clone().into_inner()
        });
        msg!("Updated global state");

        Ok(())
    }
}
