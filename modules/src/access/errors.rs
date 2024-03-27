//! Errors for Access Control module.

/// Access Control-related errors.
#[odra::odra_error]
pub enum Error {
    /// The owner is not set.
    OwnerNotSet = 20_000,
    /// The caller is not the owner.
    CallerNotTheOwner = 20_001,
    /// The caller is not the new owner.
    CallerNotTheNewOwner = 20_002,
    /// The role is missing.
    MissingRole = 20_003,
    /// The role cannot be renounced for another address.
    RoleRenounceForAnotherAddress = 20_004
}
