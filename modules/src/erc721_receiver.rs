//! A pluggable Odra module implementing Erc721Receiver.
use crate::erc721::extensions::erc721_receiver::Erc721Receiver as Erc721ReceiverTrait;
use crate::erc721_receiver::events::Received;
use crate::erc721_token::Erc721TokenContractRef;
use odra::prelude::*;
use odra::{Address, Bytes, Module, U256};

/// The ERC721 receiver implementation.
// TODO: remove {} when #295 is merged
#[odra::module]
pub struct Erc721Receiver {}

#[odra::module]
impl Erc721ReceiverTrait for Erc721Receiver {
    fn on_erc721_received(
        &mut self,
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        token_id: &U256,
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        self.env().emit_event(Received {
            operator: Some(*operator),
            from: Some(*from),
            token_id: *token_id,
            data: data.clone()
        });
        Erc721TokenContractRef {
            env: self.env(),
            address: self.env().caller()
        }
        .owner_of(*token_id)
            == self.env().self_address()
    }
}

/// Erc721Receiver-related events.
pub mod events {
    use casper_event_standard::Event;
    use odra::{Address, Bytes, U256};

    /// Emitted when the transfer is accepted by the contract.
    #[derive(Event, PartialEq, Eq, Debug, Clone)]
    pub struct Received {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_id: U256,
        pub data: Option<Bytes>
    }
}
