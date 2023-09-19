//! A pluggable Odra module implementing Erc1155Receiver.

use crate::erc1155_receiver::events::{BatchReceived, SingleReceived};
use odra::types::{
    casper_types::{bytesrepr::Bytes, U256},
    event::OdraEvent,
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
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        #[allow(unused_variables)] token_id: &U256,
        #[allow(unused_variables)] amount: &U256,
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        SingleReceived {
            operator: Some(*operator),
            from: Some(*from),
            token_id: *token_id,
            amount: *amount,
            data: data.clone()
        }
        .emit();
        true
    }

    /// This function is called at the end of a [safe_batch_transfer_from](crate::erc1155::Erc1155::safe_batch_transfer_from)
    /// after the balances have been updated. To accept the transfer(s), this must return true.
    ///
    /// Emits [BatchReceived].
    pub fn on_erc1155_batch_received(
        &mut self,
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        #[allow(unused_variables)] token_ids: &[U256],
        #[allow(unused_variables)] amounts: &[U256],
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        BatchReceived {
            operator: Some(*operator),
            from: Some(*from),
            token_ids: token_ids.to_vec(),
            amounts: amounts.to_vec(),
            data: data.clone()
        }
        .emit();
        true
    }
}

/// Erc1155Receiver-related events
pub mod events {
    use odra::prelude::vec::Vec;
    use odra::types::{
        casper_types::{bytesrepr::Bytes, U256},
        Address
    };

    /// Emitted when the transferred token is accepted by the contract.
    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct SingleReceived {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_id: U256,
        pub amount: U256,
        pub data: Option<Bytes>
    }

    /// Emitted when the transferred tokens are accepted by the contract.
    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct BatchReceived {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_ids: Vec<U256>,
        pub amounts: Vec<U256>,
        pub data: Option<Bytes>
    }
}
