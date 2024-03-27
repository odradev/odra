//! Errors implementation for the security module.

/// Errors for the security module.
#[odra::odra_error]
pub enum Error {
    /// Contract needs to be paused first.
    PausedRequired = 21_000,
    /// Contract needs to be unpaused first.
    UnpausedRequired = 21_001
}
