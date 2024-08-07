use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    errors::ContractError, state::bonding_curve::curve::BondingCurve, Global, ProgramStatus,
};

#[event]
pub struct CurveSetAuthorityEvent {
    pub prev_cex_authority: Pubkey,
    pub prev_brand_authority: Pubkey,
    pub new_cex_authority: Pubkey,
    pub new_brand_authority: Pubkey,
}

#[event_cpi]
#[derive(Accounts)]
pub struct CurveSetAuthority<'info> {
    #[account(mut)]
    creator: Signer<'info>,

    // CHECK: we dont read or write to this account
    #[account()]
    brand_authority: UncheckedAccount<'info>,
    // CHECK: we dont read or write to this account
    #[account()]
    cex_authority: UncheckedAccount<'info>,

    #[account(
        mint::decimals = global.created_mint_decimals,
        mint::authority = bonding_curve,
        mint::freeze_authority = bonding_curve
    )]
    mint: Box<Account<'info, Mint>>,

    // can be called on the curve anytime
    #[account(
        mut,
        constraint = bonding_curve.creator == creator.key() @ ContractError::InvalidCreatorAuthority,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        seeds = [Global::SEED_PREFIX.as_bytes()],
        constraint = global.initialized == true @ ContractError::NotInitialized,
        constraint = global.status == ProgramStatus::Running @ ContractError::ProgramNotRunning,
        bump,
    )]
    global: Box<Account<'info, Global>>,
}

impl<'info> CurveSetAuthority<'info> {
    pub fn handler(ctx: Context<CurveSetAuthority>) -> Result<()> {
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        let prev_cex_authority = bonding_curve.cex_authority;
        let prev_brand_authority = bonding_curve.brand_authority;
        bonding_curve.cex_authority = ctx.accounts.cex_authority.key();
        bonding_curve.brand_authority = ctx.accounts.brand_authority.key();
        emit_cpi!(CurveSetAuthorityEvent {
            prev_cex_authority,
            prev_brand_authority,
            new_cex_authority: ctx.accounts.cex_authority.key(),
            new_brand_authority: ctx.accounts.brand_authority.key(),
        });
        msg!("SetCurveAuthority: done");
        Ok(())
    }
}
