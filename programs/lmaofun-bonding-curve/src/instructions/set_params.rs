use crate::{errors::ProgramError, events::*, state::global::*};
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction( params: GlobalSettingsInput)]
pub struct SetParams<'info> {
    #[account(mut,
    constraint = authority.key() == global.global_authority.key() @ ProgramError::InvalidAuthority
    )]
    authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Global::SEED_PREFIX],
        constraint = global.initialized == true @ ProgramError::NotInitialized,
        bump,
    )]
    global: Box<Account<'info, Global>>,

    #[account()]
    /// CHECK: This is not dangerous because we don't read or write from this account
    new_authority: Option<UncheckedAccount<'info>>,

    #[account()]
    /// CHECK: This is not dangerous because we don't read or write from this account
    new_fee_recipient: Option<UncheckedAccount<'info>>,

    system_program: Program<'info, System>,
}

impl SetParams<'_> {
    pub fn handler(ctx: Context<SetParams>, params: GlobalSettingsInput) -> Result<()> {
        let global = &mut ctx.accounts.global;

        global.update_authority(GlobalAuthorityInput {
            global_authority: if let Some(new_authority) = ctx.accounts.new_authority.as_ref() {
                Some(*new_authority.key)
            } else {
                None
            },
            fee_recipient: if let Some(new_fee_recipient) = ctx.accounts.new_fee_recipient.as_ref()
            {
                Some(*new_fee_recipient.key)
            } else {
                None
            },
        });
        global.update_settings(params);

        emit_cpi!(global.into_event());
        msg!("Updated global state");

        Ok(())
    }
}
