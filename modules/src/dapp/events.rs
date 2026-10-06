//! Events emitted by the dapp modules.
use odra::prelude::*;

/// Emitted when the dapp metadata is set.
///
/// The fields of [`DappMetadata`](super::DappMetadata) are listed one by one: an event field must have a
/// plain CL type, which a custom struct does not.
#[odra::event]
pub struct DappMetadataChanged {
    /// The name of the dapp.
    pub name: String,
    /// A short description of the dapp.
    pub description: String,
    /// The URL of the dapp's website.
    pub website_url: String,
    /// The URL of the dapp's icon.
    pub icon_url: String
}

/// Emitted when a contract is added to the dapp.
#[odra::event]
pub struct DappContractAdded {
    /// The added contract.
    pub contract: Address,
    /// Whether the contract can register other contracts.
    pub is_factory: bool,
    /// The address that added the contract.
    pub registrar: Address
}

/// Emitted when a contract is removed from the dapp.
#[odra::event]
pub struct DappContractRemoved {
    /// The removed contract.
    pub contract: Address,
    /// The address that removed the contract.
    pub registrar: Address
}

/// Emitted when a dapp contract points to a new dapp registry.
#[odra::event]
pub struct DappRegistryChanged {
    /// The previous registry, if any.
    pub previous_registry: Option<Address>,
    /// The new registry.
    pub new_registry: Address
}
