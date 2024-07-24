use anchor_lang::accounts::signer;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::system_program::transfer;
use anchor_lang::{
    prelude::*,
    solana_program::system_program::{self, *},
};
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

use crate::state::global;
use crate::{
    errors::ProgramError, events::CreateEvent, state::bonding_curve::BondingCurve, Global,
    ProgramStatus,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateBondingCurveParams {
    name: String,
    symbol: String,
    uri: String,
    start_time: Option<i64>,
}

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
        constraint = global.initialized == true @ ProgramError::NotInitialized,
        constraint = global.status == ProgramStatus::Running @ ProgramError::ProgramNotRunning,
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
    pub fn handler(
        ctx: Context<CreateBondingCurve>,
        params: CreateBondingCurveParams,
    ) -> Result<()> {
        let CreateBondingCurveParams {
            name,
            symbol,
            uri,
            start_time,
        } = params;
        let creator_info = ctx.accounts.creator.to_account_info();
        let mint_info = ctx.accounts.mint.to_account_info();
        let mint_authority_info = ctx.accounts.global.to_account_info();

        let metadata_info = ctx.accounts.metadata.to_account_info();

        let bonding_curve_token_account_info =
            ctx.accounts.bonding_curve_token_account.to_account_info();

        let initial_supply = ctx.accounts.global.initial_token_supply;
        msg!(
            "create::BondingCurve::get_lamports: {:?}",
            &ctx.accounts.bonding_curve.get_lamports()
        );

        let signer = ctx.accounts.global.get_signer(&ctx.bumps.global);
        let signer_seeds = &[&signer[..]];

        let token_data: DataV2 = DataV2 {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
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
            initial_supply,
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

        //create bonding curve
        let clock = Clock::get()?;
        let pool_start_time = start_time.unwrap_or(clock.unix_timestamp);
        let bonding_curve = &mut ctx.accounts.bonding_curve.new_from_global(
            &ctx.accounts.global,
            ctx.accounts.creator.key(),
            pool_start_time,
        );

        emit_cpi!(CreateEvent {
            name,
            symbol,
            uri,
            mint: *ctx.accounts.mint.to_account_info().key,
            creator: *ctx.accounts.creator.to_account_info().key,
            virtual_sol_reserves: bonding_curve.virtual_sol_reserves,
            virtual_token_reserves: bonding_curve.virtual_token_reserves,
            token_total_supply: bonding_curve.token_total_supply,
        });

        Ok(())
    }
}
