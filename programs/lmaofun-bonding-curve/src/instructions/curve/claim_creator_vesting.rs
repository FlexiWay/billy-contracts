use crate::{
    errors::ContractError,
    state::{bonding_curve::BondingCurve, global::*},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, TokenAccount},
};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimCreatorVesting<'info> {
    #[account(mut,
    constraint = creator.key() == bonding_curve.creator.key() @ ContractError::InvalidCreatorAuthority
    )]
    creator: Signer<'info>,

    #[account(
        mut,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator,
    )]
    user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [Global::SEED_PREFIX.as_bytes()],
        constraint = global.initialized == true @ ContractError::NotInitialized,
        constraint = global.status != ProgramStatus::Paused @ ContractError::ProgramNotRunning,
        bump,
    )]
    global: Box<Account<'info, Global>>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    mint: UncheckedAccount<'info>,

    system_program: Program<'info, System>,
    clock: Sysvar<'info, Clock>,
    rent: Sysvar<'info, Rent>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
}

impl ClaimCreatorVesting<'_> {
    pub fn validate(&self) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            self.bonding_curve.is_started(&clock),
            ContractError::CurveNotStarted
        );
        Ok(())
    }
    pub fn handler(_ctx: Context<ClaimCreatorVesting>) -> Result<()> {
        Ok(())
    }
}
