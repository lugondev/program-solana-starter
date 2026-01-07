use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Invalid amount provided")]
    InvalidAmount,

    #[msg("Arithmetic overflow occurred")]
    ArithmeticOverflow,

    #[msg("Account is not rent exempt")]
    NotRentExempt,

    #[msg("Invalid state transition")]
    InvalidStateTransition,

    #[msg("CPI call failed")]
    CpiFailed,

    #[msg("Invalid mint address")]
    InvalidMint,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Program is paused")]
    ProgramPaused,
}
