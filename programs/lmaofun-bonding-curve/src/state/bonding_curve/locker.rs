use anchor_lang::prelude::*;
use anchor_lang::{prelude::AccountInfo, Accounts};
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{self, FreezeAccount, Mint, ThawAccount, Token, TokenAccount};

use crate::errors::ContractError;
use crate::state::bonding_curve::BondingCurve;
use crate::Global;

#[derive(Accounts)]
pub struct BondingCurveLockerCtx<'info> {
    #[account()]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [Global::SEED_PREFIX.as_bytes()],
        bump,
    )]
    pub global: Box<Account<'info, Global>>,
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
    pub fn lock_ata<'a>(&self, global_bump: u8) -> Result<()> {
        let signer: [&[u8]; 2] = Global::get_signer(&global_bump);
        let signer_seeds = &[&signer[..]];
        let accs = FreezeAccount {
            account: self.bonding_curve_token_account.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.global.to_account_info(),
        };
        token::freeze_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accs,
            signer_seeds,
        ))?;
        msg!("BondingCurveLockerCtx::lock_ata complete");

        Ok(())
    }
    pub fn unlock_ata<'a>(&self, global_bump: u8) -> Result<()> {
        // let mint_key = self.mint.key();
        let signer = Global::get_signer(&global_bump);
        let signer_seeds = &[&signer[..]];

        let accs = ThawAccount {
            account: self.bonding_curve_token_account.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.global.to_account_info(),
        };
        token::thaw_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accs,
            signer_seeds,
        ))?;
        msg!("BondingCurveLockerCtx::unlock_ata complete");

        Ok(())
    }

    pub fn revoke_mint_authority(&self, global_bump: u8) -> Result<()> {
        let mint_info = self.mint.to_account_info();
        let mint_authority_info = self.global.to_account_info();
        let signer = Global::get_signer(&global_bump);
        let signer_seeds = &[&signer[..]];

        //remove mint_authority
        token::set_authority(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: mint_authority_info.clone(),
                    account_or_mint: mint_info.clone(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None,
        )?;
        msg!("CreateBondingCurve::revoke_mint_authority: done");

        Ok(())
    }

    pub fn revoke_freeze_authority(&self, global_bump: u8) -> Result<()> {
        let mint_info = self.mint.to_account_info();
        let mint_authority_info = self.global.to_account_info();
        let signer = Global::get_signer(&global_bump);
        let signer_seeds = &[&signer[..]];

        // revoke freeze authority
        token::set_authority(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: mint_authority_info.clone(),
                    account_or_mint: mint_info.clone(),
                },
                signer_seeds,
            ),
            AuthorityType::FreezeAccount,
            None,
        )?;

        msg!("CreateBondingCurve::revoke_freeze_authority: done");

        Ok(())
    }
}
pub trait IntoBondingCurveLockerCtx<'info> {
    fn into_bonding_curve_locker_ctx(&self) -> BondingCurveLockerCtx<'info>;
}
