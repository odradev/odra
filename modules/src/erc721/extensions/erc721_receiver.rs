use odra::types::{Address, U256};

/// The ERC721 receiver interface.
pub trait Erc721Receiver {
    fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: Option<Vec<u8>>
    ) -> bool;
}
