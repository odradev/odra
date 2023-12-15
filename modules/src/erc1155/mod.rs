//! Erc1155 standard implementation.
use odra::prelude::*;
use odra::{Address, Bytes, U256};

pub mod erc1155_base;
pub mod extensions;
pub mod owned_erc1155;

/// The ERC-1155 interface as defined in the standard.
pub trait Erc1155 {
    /// Returns the amount of tokens of token type `id` owned by `owner`.
    fn balance_of(&self, owner: &Address, id: &U256) -> U256;
    ///  Batched version of [Erc1155::balance_of](Self::balance_of).
    ///
    /// The length of `owners` and `ids` must be the same.
    fn balance_of_batch(&self, owners: Vec<Address>, ids: Vec<U256>) -> Vec<U256>;
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
}

/// Erc1155-related Odra events.
pub mod events {
    use casper_event_standard::Event;
    use odra::prelude::*;
    use odra::{Address, U256};

    /// Emitted when a single Erc1155 transfer is performed.
    #[derive(Event, PartialEq, Eq, Debug, Clone)]
    pub struct TransferSingle {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub id: U256,
        pub value: U256
    }

    /// Emitted when a batched Erc1155 transfer is performed.
    #[derive(Event, PartialEq, Eq, Debug, Clone)]
    pub struct TransferBatch {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub ids: Vec<U256>,
        pub values: Vec<U256>
    }

    /// Emitted when the `owner` approves or revokes the `operator`.
    #[derive(Event, PartialEq, Eq, Debug, Clone)]
    pub struct ApprovalForAll {
        pub owner: Address,
        pub operator: Address,
        pub approved: bool
    }
}

/// Erc1155-related Odra errors.
pub mod errors {
    use odra::OdraError;

    /// Possible errors in the context of Erc1155.
    #[derive(OdraError)]
    pub enum Error {
        /// Collections of addresses and token ids have different length.
        AccountsAndIdsLengthMismatch = 30_000,
        /// The owner cannot approve himself.
        ApprovalForSelf = 30_001,
        /// The operator is not allowed to perform the action.
        NotAnOwnerOrApproved = 30_002,
        /// Insufficient token amount to perform a transaction.
        InsufficientBalance = 30_003,
        /// Token transfer finished with an error.
        TransferRejected = 30_004,
        /// Collections of token ids and amounts have different length.
        IdsAndAmountsLengthMismatch = 30_005
    }
}
