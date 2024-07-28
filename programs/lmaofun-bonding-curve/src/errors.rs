use anchor_lang::error_code;

#[error_code]
pub enum ProgramError {
    #[msg("Invalid Global Authority")]
    InvalidGlobalAuthority,
    #[msg("Invalid Withdraw Authority")]
    InvalidWithdrawAuthority,
    #[msg("Invalid Argument")]
    InvalidArgument,

    #[msg("Global Already Initialized")]
    AlreadyInitialized,
    #[msg("Global Not Initialized")]
    NotInitialized,

    #[msg("Not in Running State")]
    ProgramNotRunning,

    #[msg("Bonding Curve Complete")]
    BondingCurveComplete,
    #[msg("Bonding Curve Not Complete")]
    BondingCurveNotComplete,

    #[msg("Insufficient User Tokens")]
    InsufficientUserTokens,
    #[msg("Insufficient Curve Tokens")]
    InsufficientCurveTokens,

    #[msg("Insufficient user SOL")]
    InsufficientUserSOL,

    #[msg("Slippage Exceeded")]
    SlippageExceeded,

    #[msg("Swap exactInAmount is 0")]
    MinSwap,

    #[msg("Buy Failed")]
    BuyFailed,
    #[msg("Sell Failed")]
    SellFailed,

    #[msg("Bonding Curve Invariant Failed")]
    BondingCurveInvariant,

    #[msg("Curve Not Started")]
    CurveNotStarted,

    #[msg("Invalid Allocation Data supplied, percents must add up to 100")]
    InvalidAllocation,

    #[msg("Start time is in the past")]
    InvalidStartTime,
}
