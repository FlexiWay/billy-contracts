use anchor_lang::prelude::*;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod util;
use instructions::claim_creator_vesting::*;
use instructions::{
    config::{initialize::*, set_curve_authority::*},
    create_bonding_curve::*,
    initialize::*,
    set_params::*,
    swap::*,
    withdraw_fees::*,
};
use state::bonding_curve::CreateBondingCurveParams;
use state::global::*;
declare_id!("71odFTZ59cG8yyBtEZrnJdBYaepzri2A12hEc16vK6WP");

#[program]
pub mod billy_bonding_curve {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: GlobalSettingsInput) -> Result<()> {
        Initialize::handler(ctx, params)
    }
    pub fn set_params(ctx: Context<SetParams>, params: GlobalSettingsInput) -> Result<()> {
        SetParams::handler(ctx, params)
    }

    pub fn curve_config_initialize(
        ctx: Context<CurveInitialize>,
        params: CurveInitializeParams,
    ) -> Result<()> {
        CurveInitialize::handler(ctx, params)
    }
    pub fn curve_config_update_authority(ctx: Context<CurveSetAuthority>) -> Result<()> {
        CurveSetAuthority::handler(ctx)
    }

    #[access_control(ctx.accounts.validate(&params))]
    pub fn create_bonding_curve(
        ctx: Context<CreateBondingCurve>,
        params: CreateBondingCurveParams,
    ) -> Result<()> {
        CreateBondingCurve::handler(ctx, params)
    }

    #[access_control(ctx.accounts.validate(&params))]
    pub fn swap(ctx: Context<Swap>, params: SwapParams) -> Result<()> {
        Swap::handler(ctx, params)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn claim_creator_vesting(ctx: Context<ClaimCreatorVesting>) -> Result<()> {
        ClaimCreatorVesting::handler(ctx)
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        WithdrawFees::handler(ctx)
    }
}
