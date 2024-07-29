use crate::{
    errors::ContractError,
    state::{bonding_curve::BondingCurve, distributors::CreatorDistributor, global::*},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Token, TokenAccount},
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
        seeds = [CreatorDistributor::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
    )]
    creator_distributor: Box<Account<'info, CreatorDistributor>>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = creator_distributor,
    )]
    creator_distributor_token_account: Box<Account<'info, TokenAccount>>,

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

        let seconds_since_start = clock
            .unix_timestamp
            .checked_sub(self.bonding_curve.start_time)
            .unwrap();

        require!(
            seconds_since_start > self.bonding_curve.vesting_terms.cliff,
            ContractError::CliffNotReached
        );

        Ok(())
    }
    pub fn handler(ctx: Context<ClaimCreatorVesting>) -> Result<()> {
        let clock = Clock::get()?;

        let tokens_per_second = (ctx.accounts.creator_distributor.initial_vested_supply as i64)
            .checked_div(ctx.accounts.bonding_curve.vesting_terms.duration)
            .unwrap() as u64;

        msg!(
            "ClaimCreatorVesting::handler: tokens_per_second: {}",
            tokens_per_second
        );
        let start_second: i64;
        if let Some(last_distribution) = ctx.accounts.creator_distributor.last_distribution {
            msg!(
                "ClaimCreatorVesting::handler: last_distribution: {}",
                last_distribution
            );
            require!(
                clock.unix_timestamp > last_distribution,
                ContractError::VestingPeriodNotOver
            );
            start_second = last_distribution + 1;
        } else {
            start_second = ctx.accounts.bonding_curve.start_time
                + ctx.accounts.bonding_curve.vesting_terms.cliff;
        }
        msg!(
            "ClaimCreatorVesting::handler: start_second: {}",
            start_second
        );
        let seconds_since_start_second = clock.unix_timestamp.checked_sub(start_second).unwrap();
        msg!(
            "ClaimCreatorVesting::handler: seconds_since_start_second: {}",
            seconds_since_start_second
        );
        let tokens_to_distribute = tokens_per_second
            .checked_mul(seconds_since_start_second as u64)
            .unwrap();
        msg!(
            "ClaimCreatorVesting::handler: tokens_to_distribute: {}",
            tokens_to_distribute
        );
        let user_token_account = ctx.accounts.user_token_account.to_account_info();
        let creator_distributor_token_account = ctx
            .accounts
            .creator_distributor_token_account
            .to_account_info();
        let mint_k = ctx.accounts.bonding_curve.mint.key();
        let signer = CreatorDistributor::get_signer(&ctx.bumps.creator_distributor, &mint_k);
        let signer_seeds = &[&signer[..]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: creator_distributor_token_account,
                    to: user_token_account,
                    authority: ctx.accounts.creator_distributor.to_account_info(),
                },
                signer_seeds,
            ),
            tokens_to_distribute,
        )?;
        ctx.accounts.creator_distributor.last_distribution = Some(clock.unix_timestamp);
        msg!("ClaimCreatorVesting::handler: done");
        Ok(())
    }
}
