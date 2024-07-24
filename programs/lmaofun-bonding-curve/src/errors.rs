use anchor_lang::error_code;

#[error_code]
pub enum ProgramError {
    #[msg("Global Already Initialized")]
    AlreadyInitialized,
    #[msg("Global Not Initialized")]
    NotInitialized,
    #[msg("Invalid Authority")]
    InvalidAuthority,
    #[msg("Not in Running State")]
    ProgramNotRunning,

    #[msg("Invalid Argument")]
    InvalidArgument,
}
