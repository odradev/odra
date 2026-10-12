//! Errors of the dapp modules.
use odra::prelude::OdraError;

/// Dapp-related errors.
#[odra::odra_error]
pub enum Error {
    /// The address is not a contract address.
    NotAContract = 22_000,
    /// The contract is already registered. The registry itself counts as registered.
    DappContractAlreadyRegistered = 22_001,
    /// The contract is not registered.
    DappContractNotRegistered = 22_002,
    /// The contract points to a different dapp registry.
    DappRegistryMismatch = 22_003,
    /// The caller is not a registered dapp factory.
    CallerNotDappFactory = 22_004,
    /// The dapp registry has not been set.
    DappRegistryNotSet = 22_005,
    /// The dapp registry cannot remove itself.
    CannotRemoveDappRegistry = 22_006
}
