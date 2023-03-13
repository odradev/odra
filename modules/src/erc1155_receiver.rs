use crate::erc1155_receiver::events::{BatchReceived, SingleReceived};
use odra::types::{event::OdraEvent, Address, Bytes, U256};

/// The ERC1155 receiver implementation.
#[odra::module]
pub struct Erc1155Receiver {}

#[odra::module]
impl Erc1155Receiver {
    pub fn on_erc1155_received(
        &mut self,
        #[allow(unused_variables)] operator: Address,
        #[allow(unused_variables)] from: Address,
        #[allow(unused_variables)] token_id: U256,
        #[allow(unused_variables)] amount: U256,
        #[allow(unused_variables)] data: Option<Bytes>
    ) -> bool {
        SingleReceived {
            operator: Some(operator),
            from: Some(from),
            token_id,
            amount,
            data
        }
        .emit();

        true
    }

    pub fn on_erc1155_batch_received(
        &mut self,
        #[allow(unused_variables)] operator: Address,
        #[allow(unused_variables)] from: Address,
        #[allow(unused_variables)] token_ids: Vec<U256>,
        #[allow(unused_variables)] amounts: Vec<U256>,
        #[allow(unused_variables)] data: Option<Bytes>
    ) -> bool {
        BatchReceived {
            operator: Some(operator),
            from: Some(from),
            token_ids,
            amounts,
            data
        }
        .emit();

        true
    }
}

pub mod events {
    use odra::types::{Address, Bytes, U256};

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct SingleReceived {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_id: U256,
        pub amount: U256,
        pub data: Option<Bytes>
    }

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct BatchReceived {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_ids: Vec<U256>,
        pub amounts: Vec<U256>,
        pub data: Option<Bytes>
    }
}
