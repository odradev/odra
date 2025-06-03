use odra::prelude::OdraError;

/// Error enum for the CEP-18 contract.
#[odra::odra_error]
pub enum Error {
    /// Spender does not have enough balance.
    InsufficientBalance = 60001,
    /// Spender does not have enough allowance approved.
    InsufficientAllowance = 60002,
    /// Operation would cause an integer overflow.
    Overflow = 60003,
    /// The user cannot target themselves.
    CannotTargetSelfUser = 60017,
    /// The contract is in an invalid state. This error should never happen.
    InvalidState = 60100
}
