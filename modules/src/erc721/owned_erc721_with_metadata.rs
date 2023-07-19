//! Erc721 with ownership.

use odra::types::Address;

use super::{extensions::erc721_metadata::Erc721Metadata, Erc721};

/// The ERC721 interface with the Ownable and Metadata traits included manually.
pub trait OwnedErc721WithMetadata: Erc721 + Erc721Metadata {
    /// Initializes the module.
    fn init(&mut self, name: String, symbol: String, base_uri: String);

    /// If the contract's owner chooses to renounce their ownership, the contract
    /// will no longer have an owner. This means that any functions that can only
    /// be accessed by the owner will no longer be available.
    ///
    /// The function can only be called by the current owner, and it will permanently
    /// remove the owner's privileges.
    ///
    /// Emits [OwnershipTransferred](crate::access::events::OwnershipTransferred).
    fn renounce_ownership(&mut self);
    /// Transfers ownership of the module to `new_owner`. This function can only
    /// be accessed by the current contract owner.
    ///
    /// Emits [OwnershipTransferred](crate::access::events::OwnershipTransferred).
    fn transfer_ownership(&mut self, new_owner: Option<Address>);
    /// Returns the address of the current owner.
    fn owner(&self) -> Option<Address>;
}
