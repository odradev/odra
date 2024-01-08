//! Erc1155 with ownership.
use odra::prelude::*;
use odra::{Address, Bytes, U256};

/// The ERC-1155 interface with the Ownable trait included manually.
pub trait OwnedErc1155 {
    /// Initializes the module.
    fn init(&mut self);

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
    fn transfer_ownership(&mut self, new_owner: &Address);
    /// Returns the address of the current owner.
    fn owner(&self) -> Address;
    /// Returns the amount of tokens of token type `id` owned by `owner`.
    fn balance_of(&self, owner: &Address, id: &U256) -> U256;
    ///  Batched version of [Erc1155::balance_of](Self::balance_of).
    ///
    /// The length of `owners` and `ids` must be the same.
    fn balance_of_batch(&self, owners: &[Address], ids: &[U256]) -> Vec<U256>;
    /// Allows or denials the `operator` to transfer the caller’s tokens.
    ///
    /// Emits [crate::erc1155::events::ApprovalForAll].
    fn set_approval_for_all(&mut self, operator: &Address, approved: bool);
    /// Checks if the `operator` is approved to transfer `owner`'s tokens.
    fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> bool;
    /// Transfers amount tokens of token type id from `from` to `to`.
    ///
    /// Emits [TransferSingle](crate::erc1155::events::TransferSingle).
    ///
    /// If `to` refers to a smart contract, it must implement [Erc1155Receiver::on_erc1155_received](crate::erc1155_receiver::Erc1155Receiver::on_erc1155_received).
    fn safe_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        id: &U256,
        amount: &U256,
        data: &Option<Bytes>
    );
    /// Batched version of [Erc1155::safe_transfer_from](Self::safe_transfer_from).
    ///
    /// Emits [TransferBatch](crate::erc1155::events::TransferBatch).
    ///
    /// If `to` refers to a smart contract, it must implement [Erc1155Receiver::on_erc1155_batch_received](crate::erc1155_receiver::Erc1155Receiver::on_erc1155_batch_received).
    fn safe_batch_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: &Option<Bytes>
    );

    /// Mints tokens
    fn mint(&mut self, to: &Address, id: &U256, amount: &U256, data: &Option<Bytes>);

    /// Batch mints tokens
    fn mint_batch(
        &mut self,
        to: &Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: &Option<Bytes>
    );

    /// Burns tokens
    fn burn(&mut self, from: &Address, id: &U256, amount: &U256);

    /// Burns tokens in batch
    fn burn_batch(&mut self, from: &Address, ids: Vec<U256>, amounts: Vec<U256>);
}
