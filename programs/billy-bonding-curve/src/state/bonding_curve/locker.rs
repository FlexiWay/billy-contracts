use std::cell::RefMut;

use anchor_lang::prelude::*;
// use anchor_lang::{prelude::AccountInfo, Accounts};
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{self, FreezeAccount, Mint, ThawAccount, Token, TokenAccount};

use crate::errors::ContractError;
use crate::state::bonding_curve::BondingCurve;

// #[derive(Accounts)]
pub struct BondingCurveLockerCtx<'info> {
    pub bonding_curve_bump: u8,
    // #[account()]
    pub mint: Box<Account<'info, Mint>>,

    // // #[account(
    // //     mut,
    // //     seeds = [Global::SEED_PREFIX.as_bytes()],
    // //     bump,
    // // )]
    // pub global: Box<Account<'info, Global>>,
    // #[account(
    //     mut,
    //     seeds = [BondingCurve::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
    //     bump,
    // )]
    pub bonding_curve: AccountLoader<'info, BondingCurve>,
    // #[account(
    //     mut,
    //     associated_token::mint = mint,
    //     associated_token::authority = bonding_curve,
    // )]
    pub bonding_curve_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
use std::ops::DerefMut;
impl BondingCurveLockerCtx<'_> {
    fn get_signer<'a>(&self) -> [&[u8]; 3] {
        let signer: [&[u8]; 3] =
            BondingCurve::get_signer(&self.bonding_curve_bump, self.mint.to_account_info().key);
        signer
    }
    pub fn invariant<'info>(&mut self) -> Result<()> {
        let tkn_account = &mut self.bonding_curve_token_account;
        if tkn_account.owner != self.bonding_curve.key() {
            msg!("Invariant failed: invalid token acc supplied");
            return Err(ContractError::BondingCurveInvariant.into());
        }
        let lamports = self.bonding_curve.get_lamports();
        let bonding_curve_info = self.bonding_curve.to_account_info();
        let bonding_curve: RefMut<BondingCurve> = RefMut::map(
            bonding_curve_info.try_borrow_mut_data()?,
            |data: &mut &mut [u8]| {
                bytemuck::from_bytes_mut(
                    &mut data.deref_mut()[8..std::mem::size_of::<BondingCurve>() + 8],
                )
            },
        );

        tkn_account.reload()?;

        let tkn_balance = tkn_account.amount;

        let rent_exemption_balance: u64 =
            Rent::get()?.minimum_balance(8 + BondingCurve::INIT_SPACE as usize);
        let bonding_curve_pool_lamports: u64 = lamports - rent_exemption_balance;

        // Ensure real sol reserves are equal to bonding curve pool lamports
        if bonding_curve_pool_lamports != bonding_curve.real_sol_reserves {
            msg!(
                "real_sol_r:{}, bonding_lamps:{}",
                bonding_curve.real_sol_reserves,
                bonding_curve_pool_lamports
            );
            msg!("Invariant failed: real_sol_reserves != bonding_curve_pool_lamports");
            return Err(ContractError::BondingCurveInvariant.into());
        }

        // Ensure the virtual reserves are always positive
        if bonding_curve.virtual_sol_reserves <= 0 {
            msg!("Invariant failed: virtual_sol_reserves <= 0");
            return Err(ContractError::BondingCurveInvariant.into());
        }
        if bonding_curve.virtual_token_reserves <= 0 {
            msg!("Invariant failed: virtual_token_reserves <= 0");
            return Err(ContractError::BondingCurveInvariant.into());
        }

        // Ensure the token total supply is consistent with the reserves
        if bonding_curve.real_token_reserves != tkn_balance {
            msg!("Invariant failed: real_token_reserves != tkn_balance");
            msg!("real_token_reserves: {}", bonding_curve.real_token_reserves);
            msg!("tkn_balance: {}", tkn_balance);
            return Err(ContractError::BondingCurveInvariant.into());
        }

        // Ensure the bonding curve is complete only if real token reserves are zero
        if bonding_curve.complete && bonding_curve.real_token_reserves != 0 {
            msg!("Invariant failed: bonding curve marked as complete but real_token_reserves != 0");
            return Err(ContractError::BondingCurveInvariant.into());
        }

        if !bonding_curve.complete && !tkn_account.is_frozen() {
            msg!("Active BondingCurve TokenAccount must always be frozen at the end");
            return Err(ContractError::BondingCurveInvariant.into());
        }
        Ok(())
    }

    pub fn lock_ata<'a>(&self) -> Result<()> {
        let signer = self.get_signer();
        let signer_seeds: &[&[&[u8]]; 1] = &[&signer[..]];

        let accs = FreezeAccount {
            account: self.bonding_curve_token_account.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.bonding_curve.to_account_info(),
        };
        token::freeze_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accs,
            signer_seeds,
        ))?;
        msg!("BondingCurveLockerCtx::lock_ata complete");

        Ok(())
    }
    pub fn unlock_ata<'a>(&self) -> Result<()> {
        let signer = self.get_signer();
        let signer_seeds: &[&[&[u8]]; 1] = &[&signer[..]];

        let accs = ThawAccount {
            account: self.bonding_curve_token_account.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.bonding_curve.to_account_info(),
        };
        token::thaw_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accs,
            signer_seeds,
        ))?;
        msg!("BondingCurveLockerCtx::unlock_ata complete");

        Ok(())
    }

    pub fn revoke_mint_authority(&self) -> Result<()> {
        let mint_info = self.mint.to_account_info();
        let mint_authority_info = self.bonding_curve.to_account_info();
        let signer = self.get_signer();
        let signer_seeds: &[&[&[u8]]; 1] = &[&signer[..]];

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

    pub fn revoke_freeze_authority(&self) -> Result<()> {
        let mint_info = self.mint.to_account_info();
        let mint_authority_info = self.bonding_curve.to_account_info();
        let signer = self.get_signer();
        let signer_seeds: &[&[&[u8]]; 1] = &[&signer[..]];

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
    fn into_bonding_curve_locker_ctx(&self, bonding_curve_bump: u8)
        -> BondingCurveLockerCtx<'info>;
}
