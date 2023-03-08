use crate::erc721token::Erc721TokenRef;
use odra::contract_env::{caller, self_address};
use odra::types::{Address, U256};

#[odra::module]
pub struct Receiver {}

#[odra::module]
impl Receiver {
    pub fn on_erc721_received(
        &mut self,
        #[allow(unused_variables)]
        operator: Address,
        #[allow(unused_variables)]
        from: Address,
        token_id: U256,
        #[allow(unused_variables)]
        data: Option<Vec<u8>>
    ) -> bool {
        Erc721TokenRef::at(caller()).owner_of(token_id) == self_address()
    }
}
