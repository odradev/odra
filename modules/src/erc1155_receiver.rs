//! A pluggable Odra module implementing Erc1155Receiver.
use crate::erc1155_receiver::events::{BatchReceived, SingleReceived};
use odra::prelude::*;
use odra::{
    casper_types::{bytesrepr::Bytes, U256},
    Address
};

/// The ERC1155 receiver implementation.
#[odra::module(events = [SingleReceived, BatchReceived])]
pub struct Erc1155Receiver;

#[odra::module]
impl Erc1155Receiver {
    /// This function is called at the end of a [safe_transfer_from](crate::erc1155::Erc1155::safe_transfer_from),
    /// after the balance has been updated.  To accept the transfer, this must return true.
    ///
    /// Emits [SingleReceived].
    pub fn on_erc1155_received(
        &mut self,
        operator: &Address,
        from: &Address,
        token_id: &U256,
        amount: &U256,
        data: &Option<Bytes>
    ) -> bool {
        self.env().emit_event(SingleReceived {
            operator: Some(*operator),
            from: Some(*from),
            token_id: *token_id,
            amount: *amount,
            data: data.clone()
        });
        true
    }

    /// This function is called at the end of a [safe_batch_transfer_from](crate::erc1155::Erc1155::safe_batch_transfer_from)
    /// after the balances have been updated. To accept the transfer(s), this must return true.
    ///
    /// Emits [BatchReceived].
    pub fn on_erc1155_batch_received(
        &mut self,
        operator: &Address,
        from: &Address,
        token_ids: Vec<U256>,
        amounts: Vec<U256>,
        data: &Option<Bytes>
    ) -> bool {
        self.env().emit_event(BatchReceived {
            operator: Some(*operator),
            from: Some(*from),
            token_ids: token_ids.to_vec(),
            amounts: amounts.to_vec(),
            data: data.clone()
        });
        true
    }
}

/// Erc1155Receiver-related events
pub mod events {
    use odra::prelude::*;
    use odra::{
        casper_types::{bytesrepr::Bytes, U256},
        Address
    };

    /// Emitted when the transferred token is accepted by the contract.
    #[odra::event]
    pub struct SingleReceived {
        /// The operator that called the function.
        pub operator: Option<Address>,
        /// The address of the sender.
        pub from: Option<Address>,
        /// The token id.
        pub token_id: U256,
        /// The token amount.
        pub amount: U256,
        /// The token data.
        pub data: Option<Bytes>
    }

    /// Emitted when the transferred tokens are accepted by the contract.
    #[odra::event]
    pub struct BatchReceived {
        /// The operator that called the function.
        pub operator: Option<Address>,
        /// The address of the sender.
        pub from: Option<Address>,
        /// The token ids.
        pub token_ids: Vec<U256>,
        /// The token amounts.
        pub amounts: Vec<U256>,
        /// The token data.
        pub data: Option<Bytes>
    }
}
