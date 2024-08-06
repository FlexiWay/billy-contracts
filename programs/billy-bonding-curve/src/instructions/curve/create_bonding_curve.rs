use std::cell::RefMut;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::state::{
    bonding_curve::*,
    global::*,
    vaults::{BrandVault, CreatorVault, PlatformVault, PresaleVault},
};

use crate::{errors::ContractError, events::CreateEvent};

use crate::state::bonding_curve::locker::{BondingCurveLockerCtx, IntoBondingCurveLockerCtx};

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: CreateBondingCurveParams)]
pub struct CreateBondingCurve<'info> {
    #[account(
        init,
        payer = creator,
        mint::decimals = global.created_mint_decimals,
        mint::authority = bonding_curve,
        mint::freeze_authority = bonding_curve
    )]
    mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        seeds = [CreatorVault::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        space = 8 + CreatorVault::INIT_SPACE,
        bump,
    )]
    creator_vault: AccountLoader<'info, CreatorVault>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator_vault,
    )]
    creator_vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        seeds = [PresaleVault::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        space = 8 + PresaleVault::INIT_SPACE,
        bump,
    )]
    presale_vault: AccountLoader<'info, PresaleVault>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = presale_vault,
    )]
    presale_vault_token_account: Box<Account<'info, TokenAccount>>,

    // /// CHECK: This is not dangerous because we don't read or write from this account
    // #[account()]
    // brand_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = creator,
        seeds = [BrandVault::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        space = 8 + BrandVault::INIT_SPACE,
        bump,
    )]
    brand_vault: AccountLoader<'info, BrandVault>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = brand_vault,
    )]
    brand_vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        seeds = [PlatformVault::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        space = 8 + PlatformVault::INIT_SPACE,
        bump,
    )]
    platform_vault: AccountLoader<'info, PlatformVault>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = platform_vault,
    )]
    platform_vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), mint.to_account_info().key.as_ref()],
        bump,
        space = 8 + BondingCurve::INIT_SPACE,
    )]
    bonding_curve: AccountLoader<'info, BondingCurve>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    bonding_curve_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [Global::SEED_PREFIX.as_bytes()],
        constraint = global.initialized == true @ ContractError::NotInitialized,
        constraint = global.status == ProgramStatus::Running @ ContractError::ProgramNotRunning,
        bump,
    )]
    global: Box<Account<'info, Global>>,

    // #[account(
    //     init,
    //     payer = creator,
    //     associated_token::mint = mint,
    //     associated_token::authority = global,
    // )]
    // global_token_account: Box<Account<'info, TokenAccount>>,
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

impl<'info> IntoBondingCurveLockerCtx<'info> for CreateBondingCurve<'info> {
    fn into_bonding_curve_locker_ctx(
        &self,
        bonding_curve_bump: u8,
    ) -> BondingCurveLockerCtx<'info> {
        BondingCurveLockerCtx {
            bonding_curve_bump,
            mint: self.mint.clone(),
            bonding_curve: self.bonding_curve.clone(),
            bonding_curve_token_account: self.bonding_curve_token_account.clone(),
            token_program: self.token_program.clone(),
        }
    }
}
impl CreateBondingCurve<'_> {
    pub fn validate(&self, params: &CreateBondingCurveParams) -> Result<()> {
        let clock = Clock::get()?;
        // msg!(
        //     "CreateBondingCurve::validate: allocation: {:#?}",
        //     params.allocation
        // );
        // // todo complete validation for params,allocations and start time
        // require!(
        //     AllocationData::from(params.allocation).is_valid(),
        //     ContractError::InvalidAllocation
        // );

        msg!(
            "CreateBondingCurve::validate: start_time: {:#?}",
            params.start_time
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
            self.mint.key(),
            self.creator.key(),
            self.creator.key(),
            self.global.withdraw_authority.key(),
            &params,
            &clock,
            0,
        );
        match bc.get_max_attainable_sol() {
            Some(max_sol) => {
                msg!("max:{}, threshold:{}", max_sol, params.sol_launch_threshold);
                require!(
                    params.sol_launch_threshold <= max_sol,
                    ContractError::SOLLaunchThresholdTooHigh
                );
                msg!("validated");
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
        msg!("CreateBondingCurve::handler: start");
        let clock = Clock::get()?;

        let mint_k = ctx.accounts.mint.key();
        let mint_authority_signer = BondingCurve::get_signer(&ctx.bumps.bonding_curve, &mint_k);
        let mint_auth_signer_seeds = &[&mint_authority_signer[..]];
        let mint_authority_info = ctx.accounts.bonding_curve.to_account_info().clone();

        let token_program_info = ctx.accounts.token_program.to_account_info();

        let creator_vault_ata_info = ctx.accounts.creator_vault_token_account.to_account_info();
        let presale_vault_ata_info = ctx.accounts.presale_vault_token_account.to_account_info();
        let brand_vault_ata_info = ctx.accounts.brand_vault_token_account.to_account_info();
        let platform_vault_ata_info = ctx.accounts.platform_vault_token_account.to_account_info();
        let bonding_curve_ata_info = ctx.accounts.bonding_curve_token_account.to_account_info();

        let bc_creator_vested_supply: u64;
        let bc_presale_supply: u64;
        let bc_bonding_supply: u64;
        let bc_platform_supply: u64;
        let bc_lifetime_brandkit_supply: u64;
        let bc_launch_brandkit_supply: u64;
        let bc_pool_supply: u64;

        {
            // let actual_size = ctx.accounts.bonding_curve.to_account_info().data_len();
            // msg!("actual_size: {}", actual_size);
            let mut bonding_curve: RefMut<BondingCurve> = ctx.accounts.bonding_curve.load_init()?;
            msg!("CreateBondingCurve::handler: loaded");
            bonding_curve.update_from_params(
                ctx.accounts.mint.key(),
                ctx.accounts.creator.key(),
                ctx.accounts.creator.key(),
                ctx.accounts.global.withdraw_authority.key(),
                &params,
                &clock,
                ctx.bumps.bonding_curve,
            );

            // validate allocations again
            require!(
                bonding_curve.allocation.is_valid(),
                ContractError::InvalidAllocation
            );
            bc_creator_vested_supply = bonding_curve.creator_vested_supply;
            bc_presale_supply = bonding_curve.presale_supply;
            bc_bonding_supply = bonding_curve.bonding_supply;
            bc_platform_supply = bonding_curve.platform_supply;
            bc_lifetime_brandkit_supply = bonding_curve.lifetime_brandkit_supply;
            bc_launch_brandkit_supply = bonding_curve.launch_brandkit_supply;
            bc_pool_supply = bonding_curve.pool_supply;
            msg!("CreateBondingCurve::update_from_params: created bonding_curve");
        }
        {
            msg!("plm");

            msg!("tryload");
            let mut creator_vault_ref = ctx.accounts.creator_vault.load_init()?;
            let mut presale_vault_ref = ctx.accounts.presale_vault.load_init()?;
            let mut brand_vault_ref = ctx.accounts.brand_vault.load_init()?;
            let mut platform_vault_ref = ctx.accounts.platform_vault.load_init()?;

            let mint_info = ctx.accounts.mint.to_account_info();

            msg!("try_load2");

            if bc_creator_vested_supply > 0 {
                // mint creator share to creator_vault_token_account
                mint_to(
                    CpiContext::new_with_signer(
                        token_program_info.clone(),
                        MintTo {
                            authority: mint_authority_info.clone(),
                            to: creator_vault_ata_info.clone(),
                            mint: mint_info.clone(),
                        },
                        mint_auth_signer_seeds.clone().as_ref(),
                    ),
                    bc_creator_vested_supply,
                )?;

                // ctx.accounts
                //     .creator_vault
                //     .load_init()?

                creator_vault_ref.initial_vested_supply = bc_creator_vested_supply;
                msg!("CreateBondingCurve::mint_allocations:bonding_curve.creator_vested_supply minted");
            }
            msg!("try_load3");

            if bc_presale_supply > 0 {
                // mint presale share to presale_vault_token_account
                mint_to(
                    CpiContext::new_with_signer(
                        token_program_info.clone(),
                        MintTo {
                            authority: mint_authority_info.clone(),
                            to: presale_vault_ata_info.clone(),
                            mint: mint_info.clone(),
                        },
                        mint_auth_signer_seeds.clone().as_ref(),
                    ),
                    bc_presale_supply,
                )?;
                // ctx.accounts
                //     .presale_vault
                //     .load_init()?

                presale_vault_ref.initial_vested_supply = bc_presale_supply;
                msg!("CreateBondingCurve::mint_allocations:bonding_curve.presale_supply minted");
            }
            if bc_launch_brandkit_supply > 0 || bc_lifetime_brandkit_supply > 0 {
                // mint brandkit share to brand_vault_token_account
                let amount = bc_launch_brandkit_supply + bc_lifetime_brandkit_supply;
                mint_to(
                    CpiContext::new_with_signer(
                        token_program_info.clone(),
                        MintTo {
                            authority: mint_authority_info.clone(),
                            to: brand_vault_ata_info.clone(),
                            mint: mint_info.clone(),
                        },
                        mint_auth_signer_seeds.clone().as_ref(),
                    ),
                    amount,
                )?;
                // let mut brand_vault = ctx.accounts.brand_vault.load_init()?;
                brand_vault_ref.launch_brandkit_supply = bc_launch_brandkit_supply;
                brand_vault_ref.lifetime_brandkit_supply = bc_lifetime_brandkit_supply;
                brand_vault_ref.initial_vested_supply = amount;
                msg!("CreateBondingCurve::mint_allocations:bc_launch_brandkit_supply + bc_lifetime_brandkit_supply minted");
            }
            if bc_platform_supply > 0 {
                // mint platform share to platform_vault_token_account
                mint_to(
                    CpiContext::new_with_signer(
                        token_program_info.clone(),
                        MintTo {
                            authority: mint_authority_info.clone(),
                            to: platform_vault_ata_info.clone(),
                            mint: mint_info.clone(),
                        },
                        mint_auth_signer_seeds.clone().as_ref(),
                    ),
                    bc_platform_supply,
                )?;

                // ctx.accounts
                //     .platform_vault
                //     .load_init()?
                platform_vault_ref.initial_vested_supply = bc_platform_supply;
                msg!("CreateBondingCurve::mint_allocations:bc_platform_supply minted");
            }
            // mint CURVE tokens to bonding_curve_token_account
            mint_to(
                CpiContext::new_with_signer(
                    token_program_info.clone(),
                    MintTo {
                        authority: mint_authority_info.clone(),
                        to: bonding_curve_ata_info.clone(),
                        mint: mint_info.clone(),
                    },
                    mint_auth_signer_seeds.clone().as_ref(),
                ),
                bc_bonding_supply,
            )?;
            msg!("CreateBondingCurve::mint_allocations:bonding_curve.bonding_supply minted");

            // // mint RAYDIUM POOL tokens to global_token_account
            // mint_to(
            //     CpiContext::new_with_signer(
            //        token_program_info,
            //         MintTo {
            //             authority: mint_authority_info.clone(),
            //             to: global_ata_info,.clone()
            //             mint: mint_info.clone(),
            //         },
            //         mint_auth_signer_seeds.clone().as_ref(),
            //     ),
            //     bc_pool_supply,
            // )?;
            // msg!("CreateBondingCurve::mint_allocations:bonding_curve.pool_supply minted");
            msg!("CreateBondingCurve::mint_allocations: done");
        }
        ctx.accounts
            .intialize_meta(mint_auth_signer_seeds, &params)?;

        ctx.accounts.pay_launch_fee()?;

        let locker = &mut ctx
            .accounts
            .into_bonding_curve_locker_ctx(ctx.bumps.bonding_curve);
        locker.revoke_mint_authority()?;
        locker.lock_ata()?;
        locker.invariant()?;

        // BondingCurve::invariant(locker)?;
        msg!("CreateBondingCurve::handler: invariant");
        // Context::from(ctx)
        let bonding_curve = ctx.accounts.bonding_curve.load_init()?;
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
        msg!("CreateBondingCurve::handler: success");
        Ok(())
    }
    pub fn intialize_meta(
        &mut self,
        mint_auth_signer_seeds: &[&[&[u8]]; 1],
        params: &CreateBondingCurveParams,
    ) -> Result<()> {
        let mint_info = self.mint.to_account_info();
        let mint_authority_info = self.bonding_curve.to_account_info();
        let metadata_info = self.metadata.to_account_info();
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
            self.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: self.creator.to_account_info(),
                mint: mint_info.clone(),
                metadata: metadata_info.clone(),
                update_authority: mint_authority_info.clone(),
                mint_authority: mint_authority_info.clone(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
            mint_auth_signer_seeds,
        );
        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;
        msg!("CreateBondingCurve::intialize_meta: done");
        Ok(())
    }

    // pub fn mint_allocations(
    //     &self,
    //     mint_auth_signer_seeds: &[&[&[u8]]; 1],
    //     bonding_curve: RefMut<BondingCurve>,
    // ) -> Result<()> {

    //     Ok(())
    // }

    pub fn pay_launch_fee(&mut self) -> Result<()> {
        // transfer SOL to fee recipient
        // sender is signer, must go through system program
        let fee_to = &self.platform_vault;
        let fee_from = &self.creator;
        let fee_amount = self.global.launch_fee_lamports;

        let transfer_instruction =
            system_instruction::transfer(fee_from.key, &fee_to.key(), fee_amount);

        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                fee_from.to_account_info(),
                fee_to.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[],
        )?;
        msg!("CreateBondingCurve::pay_launch_fee: done");
        Ok(())
    }
}
