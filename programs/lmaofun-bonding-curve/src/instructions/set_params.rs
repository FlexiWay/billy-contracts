use crate::{errors::CurveLaunchpadError, events::SetParamsEvent, state::global::*};
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

        emit_cpi!(SetParamsEvent {
            fee_recipient: global.fee_recipient,
            withdraw_authority: global.withdraw_authority,
            initial_virtual_token_reserves: global.initial_virtual_token_reserves,
            initial_virtual_sol_reserves: global.initial_virtual_sol_reserves,
            initial_real_token_reserves: global.initial_real_token_reserves,
            initial_token_supply: global.initial_token_supply,
            fee_basis_points: global.fee_basis_points,
        });

        Ok(())
    }
}
