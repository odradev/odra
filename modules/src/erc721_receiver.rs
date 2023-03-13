use crate::erc721_receiver::events::Received;
use crate::erc721_token::Erc721TokenRef;
use odra::contract_env::{caller, self_address};
use odra::types::{event::OdraEvent, Address, Bytes, U256};

/// The ERC721 receiver implementation.
#[odra::module]
pub struct Erc721Receiver {}

#[odra::module]
impl Erc721Receiver {
    pub fn on_erc721_received(
        &mut self,
        #[allow(unused_variables)] operator: Address,
        #[allow(unused_variables)] from: Address,
        token_id: U256,
        #[allow(unused_variables)] data: Option<Bytes>
    ) -> bool {
        Received {
            operator: Some(operator),
            from: Some(from),
            token_id,
            data
        }
        .emit();
        Erc721TokenRef::at(caller()).owner_of(token_id) == self_address()
    }
}

pub mod events {
    use odra::types::{Address, Bytes, U256};

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct Received {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_id: U256,
        pub data: Option<Bytes>
    }
}
