use std::ops::Div;

use anchor_lang::{prelude::*, solana_program::system_instruction};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

use crate::{
    errors::ProgramError,
    events::*,
    state::{bonding_curve::*, global::*},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapParams {
    pub base_in: bool,
    pub exact_in_amount: u64,
    pub min_out_amount: u64,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: SwapParams)]
pub struct Swap<'info> {
    #[account(mut)]
    user: Signer<'info>,

    #[account(
        mut,
        seeds = [Global::SEED_PREFIX.as_bytes()],
        constraint = global.initialized == true @ ProgramError::NotInitialized,
        bump,
    )]
    global: Box<Account<'info, Global>>,

    mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        constraint = bonding_curve.complete == false @ ProgramError::BondingCurveComplete,
        bump,
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    bonding_curve_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    user_token_account: Box<Account<'info, TokenAccount>>,

    system_program: Program<'info, System>,

    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,

    clock: Sysvar<'info, Clock>,
}
impl Swap<'_> {
    pub fn validate(&self, params: &SwapParams) -> Result<()> {
        let SwapParams {
            base_in,
            exact_in_amount,
            min_out_amount,
        } = params;
        let clock = Clock::get()?;

        require!(
            self.bonding_curve.is_started(&clock),
            ProgramError::CurveNotStarted
        );
        require!(exact_in_amount > &0, ProgramError::MinSwap);
        Ok(())
    }
    pub fn handler(ctx: Context<Swap>, params: SwapParams) -> Result<()> {
        let SwapParams {
            base_in,
            exact_in_amount,
            min_out_amount,
        } = params;

        msg!(
            "Swap started. BaseIn: {}, AmountIn: {}, MinOutAmount: {}",
            base_in,
            exact_in_amount,
            min_out_amount
        );

        let global_state = &ctx.accounts.global;

        let sol_amount: u64;
        let token_amount: u64;
        let fee_lamports: u64;

        if base_in {
            // Sell tokens
            require!(
                ctx.accounts.user_token_account.amount >= exact_in_amount,
                ProgramError::InsufficientUserTokens,
            );

            let sell_result = &mut ctx
                .accounts
                .bonding_curve
                .apply_sell(exact_in_amount)
                .ok_or(ProgramError::SellFailed)?;

            sol_amount = sell_result.sol_amount;
            token_amount = sell_result.token_amount;
            fee_lamports = global_state.calculate_fee(sol_amount);

            msg!("SellResult: {:#?}", sell_result);
            msg!("Fee: {} SOL", fee_lamports.div(10u64.pow(9))); // lamports to SOL
            Swap::complete_sell(&ctx, sell_result.clone(), min_out_amount, fee_lamports)?;
        } else {
            // Buy tokens
            let buy_result = &mut ctx
                .accounts
                .bonding_curve
                .apply_buy(exact_in_amount)
                .ok_or(ProgramError::BuyFailed)?;

            sol_amount = buy_result.sol_amount;
            token_amount = buy_result.token_amount;
            fee_lamports = global_state.calculate_fee(exact_in_amount);
            msg!("Fee: {} lamports", fee_lamports);

            msg!("BuyResult: {:#?}", buy_result);
            Swap::complete_buy(&ctx, buy_result.clone(), min_out_amount, fee_lamports)?;
        }
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        emit_cpi!(TradeEvent {
            mint: *ctx.accounts.mint.to_account_info().key,
            sol_amount: sol_amount,
            token_amount: token_amount,
            fee_lamports: fee_lamports,
            is_buy: !base_in,
            user: *ctx.accounts.user.to_account_info().key,
            timestamp: Clock::get()?.unix_timestamp,
            virtual_sol_reserves: bonding_curve.virtual_sol_reserves,
            virtual_token_reserves: bonding_curve.virtual_token_reserves,
            real_sol_reserves: bonding_curve.real_sol_reserves,
            real_token_reserves: bonding_curve.real_token_reserves,
        });

        BondingCurve::invariant(bonding_curve)?;

        if bonding_curve.real_token_reserves == 0 {
            bonding_curve.complete = true;

            emit_cpi!(CompleteEvent {
                user: *ctx.accounts.user.to_account_info().key,
                mint: *ctx.accounts.mint.to_account_info().key,
                virtual_sol_reserves: bonding_curve.virtual_sol_reserves,
                virtual_token_reserves: bonding_curve.virtual_token_reserves,
                real_sol_reserves: bonding_curve.real_sol_reserves,
                real_token_reserves: bonding_curve.real_token_reserves,
                timestamp: Clock::get()?.unix_timestamp,
            });
        }

        // msg!("{:#?}", bonding_curve);

        Ok(())
    }

    pub fn complete_buy(
        ctx: &Context<Swap>,
        buy_result: BuyResult,
        min_out_amount: u64,
        fee_lamports: u64,
    ) -> Result<()> {
        let bonding_curve = &ctx.accounts.bonding_curve;

        // Buy tokens
        let buy_amount_with_fee = buy_result.sol_amount + fee_lamports;

        require!(
            buy_result.token_amount >= min_out_amount,
            ProgramError::SlippageExceeded,
        );

        require!(
            ctx.accounts.user.get_lamports() >= buy_amount_with_fee,
            ProgramError::InsufficientUserSOL,
        );

        // Transfer SOL to bonding curve
        let transfer_instruction = system_instruction::transfer(
            ctx.accounts.user.key,
            bonding_curve.to_account_info().key,
            buy_result.sol_amount,
        );

        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.user.to_account_info(),
                bonding_curve.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;
        msg!("SOL to bonding curve transfer complete");
        // Transfer SOL to fee recipient
        let fee_transfer_instruction = system_instruction::transfer(
            ctx.accounts.user.key,
            &ctx.accounts.global.key(),
            fee_lamports,
        );

        anchor_lang::solana_program::program::invoke_signed(
            &fee_transfer_instruction,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.global.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;
        msg!("Fee transfer complete");

        // Transfer tokens to user
        let cpi_accounts = Transfer {
            from: ctx.accounts.bonding_curve_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };

        let signer: [&[&[u8]]; 1] = [&[
            BondingCurve::SEED_PREFIX.as_bytes(),
            ctx.accounts.mint.to_account_info().key.as_ref(),
            &[ctx.bumps.bonding_curve],
        ]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                &signer,
            ),
            buy_result.token_amount,
        )?;
        msg!("Token transfer complete");
        Ok(())
    }

    pub fn complete_sell(
        ctx: &Context<Swap>,
        sell_result: SellResult,
        min_out_amount: u64,
        fee_lamports: u64,
    ) -> Result<()> {
        // Sell tokens
        let sell_amount_minus_fee = sell_result.sol_amount - fee_lamports;
        require!(
            sell_amount_minus_fee >= min_out_amount,
            ProgramError::SlippageExceeded,
        );

        let bonding_curve = &ctx.accounts.bonding_curve;
        // Transfer tokens to bonding curve
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.bonding_curve_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            sell_result.token_amount,
        )?;
        msg!("Token to bonding curve transfer complete");
        // Transfer SOL to user
        bonding_curve.sub_lamports(sell_amount_minus_fee).unwrap();
        ctx.accounts
            .user
            .add_lamports(sell_amount_minus_fee)
            .unwrap();
        msg!("SOL to user transfer complete");
        // Transfer accrued fee to the global account
        bonding_curve.sub_lamports(fee_lamports).unwrap();
        ctx.accounts.global.add_lamports(fee_lamports).unwrap();
        msg!("Fee to global transfer complete");
        Ok(())
    }
}
