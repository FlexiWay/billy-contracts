use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{
        self, mint_to, spl_token::instruction::AuthorityType, Mint, MintTo, Token, TokenAccount,
    },
};

use crate::state::{bonding_curve::*, global::*};

use crate::{errors::ContractError, events::CreateEvent};

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: CreateBondingCurveParams)]
pub struct CreateBondingCurve<'info> {
    #[account(
        init,
        payer = creator,
        mint::decimals = global.created_mint_decimals,
        mint::authority = global,
        mint::freeze_authority = global
    )]
    mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    creator: Signer<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account()]
    brand_authority: UncheckedAccount<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account()]
    platform_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
        space = 8 + BondingCurve::INIT_SPACE,
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    bonding_curve_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [Global::SEED_PREFIX.as_bytes()],
        constraint = global.initialized == true @ ContractError::NotInitialized,
        constraint = global.status == ProgramStatus::Running @ ContractError::ProgramNotRunning,
        bump,
    )]
    global: Box<Account<'info, Global>>,

    ///CHECK: Using seed to validate metadata account
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key.as_ref(),
            mint.to_account_info().key.as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    metadata: AccountInfo<'info>,

    system_program: Program<'info, System>,

    token_program: Program<'info, Token>,

    associated_token_program: Program<'info, AssociatedToken>,

    token_metadata_program: Program<'info, Metaplex>,

    rent: Sysvar<'info, Rent>,

    clock: Sysvar<'info, Clock>,
}

impl CreateBondingCurve<'_> {
    pub fn validate(&self, params: &CreateBondingCurveParams) -> Result<()> {
        let clock = Clock::get()?;

        // todo complete validation for params,allocations and start time
        require!(
            params.allocation.is_valid(),
            ContractError::InvalidAllocation
        );

        // validate start time
        if let Some(start_time) = params.start_time {
            require!(
                start_time <= clock.unix_timestamp,
                ContractError::InvalidStartTime
            )
        }
        // validate sol_launch_threshold
        let mut d = BondingCurve::default();
        let bc = d.update_from_params(
            self.creator.key(),
            self.brand_authority.key(),
            self.platform_authority.key(),
            &params,
            &clock,
        );
        match bc.get_max_attainable_sol() {
            Some(max_sol) => {
                msg!("max:{}, thresh:{}", max_sol, params.sol_launch_threshold);
                require!(
                    params.sol_launch_threshold <= max_sol,
                    ContractError::SOLLaunchThresholdTooHigh
                )
            }
            None => {
                return Err(ContractError::NoMaxAttainableSOL.into());
            }
        }
        Ok(())
    }
    pub fn handler(
        ctx: Context<CreateBondingCurve>,
        params: CreateBondingCurveParams,
    ) -> Result<()> {
        let clock = Clock::get()?;
        ctx.accounts.bonding_curve.update_from_params(
            ctx.accounts.creator.key(),
            ctx.accounts.brand_authority.key(),
            ctx.accounts.platform_authority.key(),
            &params,
            &clock,
        );

        msg!("CreateBondingCurve::update_from_params");
        // msg!("{:#?}", bonding_curve);

        let creator_info = ctx.accounts.creator.to_account_info();
        let mint_info = ctx.accounts.mint.to_account_info();
        let mint_authority_info = ctx.accounts.global.to_account_info();

        let metadata_info = ctx.accounts.metadata.to_account_info();

        let bonding_curve_token_account_info =
            ctx.accounts.bonding_curve_token_account.to_account_info();

        let signer = Global::get_signer(&ctx.bumps.global);
        let signer_seeds = &[&signer[..]];

        let token_data: DataV2 = DataV2 {
            name: params.name.clone(),
            symbol: params.symbol.clone(),
            uri: params.uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: creator_info.clone(),
                mint: mint_info.clone(),
                metadata: metadata_info.clone(),

                update_authority: mint_authority_info.clone(),
                mint_authority: mint_authority_info.clone(),

                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        );

        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;

        //mint tokens to bonding_curve_token_account
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: mint_authority_info.clone(),
                    to: bonding_curve_token_account_info.clone(),
                    mint: mint_info.clone(),
                },
                signer_seeds,
            ),
            ctx.accounts.bonding_curve.bonding_supply,
        )?;

        //remove mint_authority
        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: mint_authority_info.clone(),
                    account_or_mint: mint_info.clone(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None,
        )?;

        // revoke freeze authority
        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: mint_authority_info.clone(),
                    account_or_mint: mint_info.clone(),
                },
                signer_seeds,
            ),
            AuthorityType::FreezeAccount,
            None,
        )?;

        // transfer SOL to fee recipient
        // sender is signer, must go through system program
        let fee_to = &ctx.accounts.global;
        let fee_from = &ctx.accounts.creator;
        let fee_amount = ctx.accounts.global.launch_fee_lamports;

        let transfer_instruction =
            system_instruction::transfer(fee_from.key, &fee_to.key(), fee_amount);

        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                fee_from.to_account_info(),
                fee_to.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        let bonding_curve = &ctx.accounts.bonding_curve;
        emit_cpi!(CreateEvent {
            name: params.name,
            symbol: params.symbol,
            uri: params.uri,
            mint: *ctx.accounts.mint.to_account_info().key,
            creator: *ctx.accounts.creator.to_account_info().key,

            virtual_sol_reserves: bonding_curve.virtual_sol_reserves,
            virtual_token_reserves: bonding_curve.virtual_token_reserves,

            token_total_supply: bonding_curve.token_total_supply,
            sol_launch_threshold: bonding_curve.sol_launch_threshold,

            real_sol_reserves: bonding_curve.real_sol_reserves,
            real_token_reserves: bonding_curve.real_token_reserves,

            start_time: bonding_curve.start_time,
        });

        Ok(())
    }
}
