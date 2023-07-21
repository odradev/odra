//! Erc721 standard implementation.

pub mod erc721_base;
pub mod extensions;
pub mod owned_erc721_with_metadata;

use odra::types::{Address, Bytes, U256};

/// The ERC-721 interface as defined in the standard.
pub trait Erc721 {
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
}

/// Erc721-related Odra events.
pub mod events {
    use odra::types::{Address, U256};

    /// Emitted when the `token_id` token is transferred (also minted or burned).
    #[derive(odra::Event)]
    pub struct Transfer {
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub token_id: U256
    }

    /// Emitted when the `owner` approves `approved` to operate on the `token_id` token.
    #[derive(odra::Event)]
    pub struct Approval {
        pub owner: Address,
        pub approved: Option<Address>,
        pub token_id: U256
    }

    /// Emitted when the `owner` approves or revokes `operator`.
    #[derive(odra::Event)]
    pub struct ApprovalForAll {
        pub owner: Address,
        pub operator: Address,
        pub approved: bool
    }
}

/// Erc721-related Odra errors.
pub mod errors {
    use odra::execution_error;

    execution_error! {
        /// Possible errors in the context of Erc721 token.
        pub enum Error {
            /// Token is invalid in the given context or does not exist.
            InvalidTokenId => 30_000,
            /// Address in not eligible to operate on the token.
            NotAnOwnerOrApproved => 30_001,
            /// The owner cannot be approved.
            ApprovalToCurrentOwner => 30_002,
            /// The caller cannot approve self.
            ApproveToCaller => 30_003,
            /// Token transfer ends with an error
            TransferFailed => 30_004,
        }
    }
}
