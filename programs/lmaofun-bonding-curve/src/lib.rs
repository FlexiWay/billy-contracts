use anchor_lang::prelude::*;

declare_id!("898RNYePTRDQaQCdvVfZdPo82vekCJXyLfc2XsWUZVx5");

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct TestState {
    pub name: u8,
    pub symbol: u8,
    pub decimals: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        mut,
        seeds = [
        "test-state".as_bytes()
        ],
        bump
        )]
    pub state: Account<'info, TestState>,
}
