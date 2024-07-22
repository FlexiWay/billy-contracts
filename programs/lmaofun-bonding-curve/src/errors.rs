use anchor_lang::error_code;

#[error_code]
pub enum CurveLaunchpadError {
    #[msg("Global Already Initialized")]
    AlreadyInitialized,
    #[msg("Global Not Initialized")]
    NotInitialized,
    #[msg("Invalid Authority")]
    InvalidAuthority,
    #[msg("Not in Running State")]
    ProgramNotRunning,
}
