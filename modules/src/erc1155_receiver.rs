use odra::types::{Address, Bytes, U256};

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
        // TODO: Fix this
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
        // TODO: Fix this
        true
    }
}
