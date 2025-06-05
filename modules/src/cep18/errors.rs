use odra::prelude::OdraError;

/// Error enum for the CEP-18 contract.
#[odra::odra_error]
pub enum Error {
    /// Spender does not have enough balance.
    InsufficientBalance = 60001,
    /// Spender does not have enough allowance approved.
    InsufficientAllowance = 60002,
    /// The user cannot target themselves.
    CannotTargetSelfUser = 60003
}
