use anchor_lang::prelude::*;
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

use crate::{
    errors::ProgramError, events::CreateEvent, state::bonding_curve::BondingCurve, Global,
    ProgramStatus,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateBondingCurveParams {
    name: String,
    symbol: String,
    uri: String,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: CreateBondingCurveParams)]
pub struct CreateBondingCurve<'info> {
    #[account(
        init,
        payer = authority,
        mint::decimals = global.created_mint_decimals,
        mint::authority = global,
        mint::freeze_authority = global
    )]
    mint: Account<'info, Mint>,

    #[account(mut)]
    authority: Signer<'info>,

    // /// CHECK: Using seed to validate mint_authority account
    // #[account(
    //     seeds=[b"mint-authority"],
    //     bump,
    // )]
    // mint_authority: AccountInfo<'info>,
    #[account(
        init,
        payer = authority,
        seeds = [BondingCurve::SEED_PREFIX, mint.to_account_info().key.as_ref()],
        bump,
        space = 8 + BondingCurve::INIT_SPACE,
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    bonding_curve_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [Global::SEED_PREFIX],
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
}

impl CreateBondingCurve<'_> {
    pub fn handler(
        ctx: Context<CreateBondingCurve>,
        params: CreateBondingCurveParams,
    ) -> Result<()> {
        let CreateBondingCurveParams { name, symbol, uri } = params;
        let creator_info = ctx.accounts.authority.to_account_info();
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

        let seeds = &["global".as_bytes(), &[ctx.bumps.global]];
        let signer = [&seeds[..]];

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
            &signer,
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
                &signer,
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
                &signer,
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
                &signer,
            ),
            AuthorityType::FreezeAccount,
            None,
        )?;

        let bonding_curve = &mut ctx
            .accounts
            .bonding_curve
            .new_from_global(&ctx.accounts.global);

        emit_cpi!(CreateEvent {
            name,
            symbol,
            uri,
            mint: *ctx.accounts.mint.to_account_info().key,
            creator: *ctx.accounts.authority.to_account_info().key,
            virtual_sol_reserves: bonding_curve.virtual_sol_reserves,
            virtual_token_reserves: bonding_curve.virtual_token_reserves,
            token_total_supply: bonding_curve.token_total_supply,
        });

        Ok(())
    }
}
