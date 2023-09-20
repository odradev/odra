//! A pluggable Odra module implementing Erc721Receiver.

use crate::erc721_receiver::events::Received;
use crate::erc721_token::Erc721TokenRef;
use odra::contract_env::{caller, self_address};
use odra::types::casper_types::bytesrepr::Bytes;
use odra::types::U256;
use odra::types::{event::OdraEvent, Address};

/// The ERC721 receiver implementation.
#[odra::module]
pub struct Erc721Receiver;

#[odra::module]
impl erc721::extensions::erc721_receiver::Erc721Receiver for Erc721Receiver {
    pub fn on_erc721_received(
        &mut self,
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        token_id: &U256,
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        Received {
            operator: Some(*operator),
            from: Some(*from),
            token_id: *token_id,
            data: data.clone()
        }
        .emit();
        Erc721TokenRef::at(&caller()).owner_of(token_id) == self_address()
    }
}

/// Erc721Receiver-related events.
pub mod events {
    use odra::types::{
        casper_types::{bytesrepr::Bytes, U256},
        Address
    };

    /// Emitted when the transfer is accepted by the contract.
    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct Received {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_id: U256,
        pub data: Option<Bytes>
    }
}
