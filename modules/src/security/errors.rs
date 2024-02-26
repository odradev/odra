//! Errors implementation for the security module.
use odra::OdraError;

/// Errors for the security module.
#[derive(OdraError)]
pub enum Error {
    /// Contract needs to be paused first.
    PausedRequired = 21_000,
    /// Contract needs to be unpaused first.
    UnpausedRequired = 21_001
}
