use crate::{errors::CurveLaunchpadError, state::global::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(params: GlobalSettingsInput)]
pub struct Initialize<'info> {
    #[account(mut)]
    global_authority: Signer<'info>,

    #[account(
        init,
        space = 8 + Global::INIT_SPACE,
        seeds = [Global::SEED_PREFIX],
        bump,
        payer = global_authority,
    )]
    global: Box<Account<'info, Global>>,

    system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn handler(ctx: Context<Initialize>, params: GlobalSettingsInput) -> Result<()> {
        let global = &mut ctx.accounts.global;

        require!(!global.initialized, CurveLaunchpadError::AlreadyInitialized);

        let authority_key = *ctx.accounts.global_authority.to_account_info().key;
        global.update_authority(GlobalAuthorityInput {
            global_authority: authority_key,
            fee_recipient: authority_key,
            withdraw_authority: authority_key,
        });
        global.update_settings(params);

        global.status = ProgramStatus::Running;
        global.initialized = true;
        msg!("Initialized global state");

        Ok(())
    }
}
