use anchor_lang::prelude::*;
use anchor_lang::{prelude::AccountInfo, Accounts};
use anchor_spl::token::{self, FreezeAccount, Mint, ThawAccount, Token, TokenAccount};

use super::BondingCurve;
#[derive(Accounts)]
pub struct BondingCurveLockerCtx<'info> {
    #[account()]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
    )]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub bonding_curve_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
impl BondingCurveLockerCtx<'_> {
    pub fn lock_ata<'a>(ctx: Context<BondingCurveLockerCtx<'a>>) {
        let mint_key = ctx.accounts.mint.key();
        let signer = BondingCurve::get_signer(&ctx.bumps.bonding_curve, &mint_key);
        let signer_seeds = &[&signer[..]];

        let accs = FreezeAccount {
            account: ctx.accounts.bonding_curve_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.bonding_curve.to_account_info(),
        };
        token::freeze_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accs,
            signer_seeds,
        ))
        .unwrap();
    }
    pub fn unlock_ata<'a>(ctx: Context<BondingCurveLockerCtx<'a>>) {
        let mint_key = ctx.accounts.mint.key();
        let signer = BondingCurve::get_signer(&ctx.bumps.bonding_curve, &mint_key);
        let signer_seeds = &[&signer[..]];

        let accs = ThawAccount {
            account: ctx.accounts.bonding_curve_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.bonding_curve.to_account_info(),
        };
        token::thaw_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accs,
            signer_seeds,
        ))
        .unwrap();
    }
}
pub trait IntoBondingCurveLockerCtx<'info> {
    fn into_bonding_curve_locker_ctx(&self) -> BondingCurveLockerCtx<'info>;
}
