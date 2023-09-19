//! Erc1155 with ownership.
use odra::types::{casper_types::U256, Address};

use super::Erc1155;

/// The ERC-1155 interface with the Ownable trait included manually.
pub trait OwnedErc1155: Erc1155 {
    /// Initializes the module.
    fn init(&mut self);

    /// Same as [Erc1155::safe_batch_transfer_from](crate::erc1155::Erc1155::safe_batch_transfer_from), does not verify if `to` implements
    /// [Erc1155Receiver::on_erc1155_batch_received](crate::erc1155_receiver::Erc1155Receiver::on_erc1155_batch_received).
    ///
    /// Emits [TransferBatch](crate::erc1155::events::TransferBatch).
    fn batch_transfer_from(&mut self, from: &Address, to: &Address, ids: &[U256], amounts: &[U256]);

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
