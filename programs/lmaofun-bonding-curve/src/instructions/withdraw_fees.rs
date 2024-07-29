use anchor_lang::{prelude::*};
use anchor_spl::token::Token;

use crate::{errors::ProgramError, events::WithdrawEvent};

use crate::state::global::*;

#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut)]
    authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Global::SEED_PREFIX.as_bytes()],
        constraint = global.withdraw_authority == *authority.key @ ProgramError::InvalidWithdrawAuthority,
        constraint = global.initialized == true @ ProgramError::NotInitialized,
        bump,
    )]
    global: Box<Account<'info, Global>>,

    system_program: Program<'info, System>,

    token_program: Program<'info, Token>,
}

impl WithdrawFees<'_> {
    pub fn handler(ctx: Context<WithdrawFees>) -> Result<()> {
        // transer sol to withdraw authority from global account
        // sender is PDA, can use lamport utilities

        let from = &ctx.accounts.global;
        let to = &ctx.accounts.authority;

        let min_balance = Rent::get()?.minimum_balance(8 + Global::INIT_SPACE as usize);

        let amount = from.get_lamports() - min_balance;

        from.sub_lamports(amount)?;
        to.add_lamports(amount)?;

        emit_cpi!(WithdrawEvent {
            withdraw_authority: *ctx.accounts.authority.key,
            amount: amount,
        });

        Ok(())
    }
}
