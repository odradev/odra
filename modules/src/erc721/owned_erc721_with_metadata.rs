//! Erc721 with ownership.
use odra::prelude::*;
use odra::{Address, Bytes, U256};

/// The ERC721 interface with the Ownable and Metadata traits included manually.
pub trait OwnedErc721WithMetadata {
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
    fn transfer_ownership(&mut self, new_owner: &Address);
    /// Returns the address of the current owner.
    fn owner(&self) -> Address;
    /// Returns the amount of tokens owned by `owner`.
    fn balance_of(&self, owner: &Address) -> U256;
    /// Returns the `owner` of the `token_id` token.
    ///
    /// Reverts if token_id does not exist.
    fn owner_of(&self, token_id: &U256) -> Address;
    /// Safely transfers `token_id` token from `from` to `to`, checking the recipient contract uses
    /// [Erc721Receiver](crate::erc721_receiver::Erc721Receiver).
    ///
    /// Emits a [Transfer](crate::erc721::events::Transfer) event.
    fn safe_transfer_from(&mut self, from: &Address, to: &Address, token_id: &U256);
    /// Safely transfers `token_id` token from `from` to `to`, checking the recipient contract uses
    /// [Erc721Receiver](crate::erc721_receiver::Erc721Receiver), passes additional data.
    ///
    /// Emits a [Transfer](crate::erc721::events::Transfer) event.
    fn safe_transfer_from_with_data(
        &mut self,
        from: &Address,
        to: &Address,
        token_id: &U256,
        data: &Bytes
    );
    fn transfer_from(&mut self, from: &Address, to: &Address, token_id: &U256);
    /// Grants permission to `approved` to transfer `token_id` token. The approval is cleared when the token is transferred.
    ///
    /// Only a single account can be approved at a time, so approving None clears the previous approval.
    fn approve(&mut self, approved: &Option<Address>, token_id: &U256);
    /// Approves or removes operator for the caller. Operators can call `transfer_from` or `safe_transfer_from` for all
    /// tokens owned by the caller.
    ///
    /// The operator cannot be the caller.
    ///
    /// Emits a [ApprovalForAll](crate::erc721::events::ApprovalForAll) event.
    fn set_approval_for_all(&mut self, operator: &Address, approved: bool);
    /// Returns the address approved for `token_id` token.
    ///
    /// Reverts if token_id does not exist.
    fn get_approved(&self, token_id: &U256) -> Option<Address>;
    /// Returns if the `operator` is allowed to manage all of the tokens of `owner`.
    fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> bool;
    /// Returns the token collection name.
    fn name(&self) -> String;
    /// Returns the token collection symbol.
    fn symbol(&self) -> String;
    /// Returns the base URI for the token collection.
    fn base_uri(&self) -> String;
    /// Mint a token and assigns it to `to`.
    fn mint(&mut self, to: &Address, token_id: &U256);
    /// Burns token.
    fn burn(&mut self, token_id: &U256);
}
